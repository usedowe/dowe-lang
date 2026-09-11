struct PreparedHarnessTurn {
    session_lock: super::lock::DataLock,
    role: HarnessRole,
    active: ModelSelection,
    explicit: Option<ModelSelection>,
    selected: ModelSelection,
    prompt: String,
    semantic: SemanticEnrichment,
    image_selection: ModelSelection,
    tools: HarnessTools,
    turn_codegraph_binding: Option<CodeGraphBinding>,
    expected_codegraph_binding: Option<CodeGraphBinding>,
    ui_task: bool,
}

async fn prepare_harness_turn(
    store: &HarnessStore,
    session: &mut HarnessSession,
    config: &HarnessConfig,
    host: &mut impl HarnessHost,
    persist: bool,
    task: super::HarnessTask<'_>,
) -> AgentResult<PreparedHarnessTurn> {
    let super::HarnessTask {
        prompt,
        role,
        active,
        explicit,
        image_paths,
        edit_scope,
        expected_codegraph_binding,
    } = task;
    config.validate()?;
    // Keep the session lease for the whole provider/tool turn. Acquiring it
    // only inside preparation would let a concurrent task start as soon as
    // the request was built, while the first turn was still running.
    let session_lock = store.lease_session(&session.id)?;
    if session.interrupted {
        return Err(AgentError::new(
            "session has an interrupted operation; inspect its events and start a new session without replaying tools",
        ));
    }
    let selected = config.resolve(role, explicit, active)?;
    // Refresh ordinary sessions at the task boundary. Orchestrated sessions
    // retain their baseline binding until the coordinator advances it.
    let graph_snapshot = if expected_codegraph_binding.is_none() {
        ensure_persistent_codegraph(store.root())
            .ok()
            .or_else(|| read_persistent_codegraph(store.root()).ok())
    } else {
        read_persistent_codegraph(store.root()).ok()
    };
    // Capture the persisted graph once; receipts retain this turn-start binding.
    let turn_codegraph_binding = graph_snapshot.as_ref().and_then(|snapshot| {
        Some(CodeGraphBinding {
            generation: snapshot.generation.clone()?,
            revision: snapshot.manifest.revision,
            root: snapshot.manifest.root.clone(),
            mode: snapshot.manifest.mode.clone(),
        })
    });
    if let Some(expected) = expected_codegraph_binding.as_ref()
        && turn_codegraph_binding.as_ref() != Some(expected)
    {
        return Err(AgentError::new(
            "persisted CodeGraphBinding does not match the expected orchestration binding",
        ));
    }
    let semantic = if matches!(role, HarnessRole::Execute | HarnessRole::Plan)
        && config.roles.contains_key(&HarnessRole::Codegraph)
    {
        match graph_snapshot.as_ref() {
            Some(snapshot) => {
                enrich_codegraph(
                    store.root(),
                    &config.resolve(HarnessRole::Codegraph, None, active)?,
                    &snapshot,
                )
                .await
            }
            None => SemanticEnrichment {
                status: SemanticStatus::Unavailable,
                provider: None,
                model: None,
                context: "[]".into(),
                usage: None,
                cache_warning: None,
            },
        }
    } else {
        SemanticEnrichment {
            status: SemanticStatus::Disabled,
            provider: None,
            model: None,
            context: "[]".into(),
            usage: None,
            cache_warning: None,
        }
    };
    let image_selection = config.resolve(HarnessRole::ImageGeneration, None, active)?;
    let historical_images = role == HarnessRole::Execute && session.turns.iter().skip(session.context_start).any(|turn| {
        turn.message.as_ref().is_some_and(|message| matches!(&message.content, AgentMessageContent::Parts(parts) if parts.iter().any(|part| matches!(part, crate::AgentMessagePart::ImageUrl { .. }))))
    });
    config.require_capabilities(
        &selected,
        role,
        !image_paths.is_empty() || historical_images,
    )?;
    let mut tools =
        HarnessTools::with_scope(store.root(), &session.id, config.clone(), edit_scope)?;
    tools.set_supervisor(host.supervisor()?);
    tools.restore_loaded_skills(&session.turns[session.context_start..]);
    for secret in host.secrets() {
        tools.redactor.add(&secret);
    }
    let prompt = tools.redactor.text(prompt);
    let ui_task = role == HarnessRole::Execute
        && (crate::skills::is_ui_authoring_prompt(&prompt)
            || !image_paths.is_empty()
            || historical_images);
    let mut required_skills = select_units(&prompt, &[]);
    let mut preloaded_skills = Vec::new();
    if ui_task && crate::is_dowe_project(store.root()) {
        required_skills.extend([
            "theme".to_string(),
            "views".to_string(),
            "views/layouts".to_string(),
            "views/pages".to_string(),
            "views/components".to_string(),
        ]);
        // Keep vector guidance available for references containing a mark or
        // icon without forcing the model to redraw it as a bitmap.
        if prompt.to_ascii_lowercase().contains("svg")
            || prompt.to_ascii_lowercase().contains("logo")
            || prompt.to_ascii_lowercase().contains("icon")
            || !image_paths.is_empty()
        {
            required_skills.push("views/svg".to_string());
        }
        required_skills.sort();
        required_skills.dedup();
        preloaded_skills = required_skills.clone();
        tools.set_required_skills(required_skills.clone())?;
        tools.set_reference_images(image_paths)?;
    }
    let user = HarnessTurn {
        message: Some(AgentMessage {
            role: "user".into(),
            content: if image_paths.is_empty() {
                AgentMessageContent::Text(prompt.clone())
            } else {
                let mut parts = vec![crate::AgentMessagePart::Text {
                    text: prompt.clone(),
                }];
                for image in crate::encode_image_paths(image_paths)? {
                    parts.push(crate::AgentMessagePart::ImageUrl {
                        image_url: crate::ImageUrl {
                            url: image.data_url,
                        },
                    });
                }
                AgentMessageContent::Parts(parts)
            },
        }),
        ..Default::default()
    };
    session.turns.push(user);
    let task_id = super::identifier();
    let baseline = super::request::task_baseline(store.root());
    emit_persist(
        store,
        session,
        persist,
        host,
        json!({"event":"task_started","taskId":task_id,"role":role,"baseline":baseline,"preloadedSkills":preloaded_skills,"visualAuthoring":ui_task}),
    )?;
    if matches!(role, HarnessRole::Codegraph | HarnessRole::Research)
        && let Some(snapshot) = graph_snapshot.as_ref()
    {
        let indexed_files = snapshot
            .graph
            .nodes
            .iter()
            .filter(|node| node.path.as_deref().is_some_and(|path| path != "."))
            .count();
        let unknown_language_files = snapshot
            .graph
            .nodes
            .iter()
            .filter(|node| {
                node.path.as_deref().is_some_and(|path| path != ".")
                    && node.language == "unknown"
            })
            .count();
        emit_persist(
            store,
            session,
            persist,
            host,
            json!({
                "event":"codegraph_coverage",
                "mode":snapshot.manifest.mode,
                "indexed_files":indexed_files,
                "unknown_language_files":unknown_language_files,
                "relationships":"deterministic_edges_only",
                "semantic_enrichment":"advisory_and_optional",
            }),
        )?;
    }
    if expected_codegraph_binding.is_none() {
        match super::refresh_capability_map(store.root()) {
            Ok(paths) if !paths.is_empty() => emit_persist(
                store,
                session,
                persist,
                host,
                json!({"event":"capability_map_stale","paths":paths}),
            )?,
            Ok(_) => {}
            Err(error) => emit_persist(
                store,
                session,
                persist,
                host,
                json!({"event":"capability_map_warning","message":error.to_string()}),
            )?,
        }
        if !store.root().join(".agents/capabilities/index.md").is_file() {
            match super::bootstrap_capability_map(store.root()) {
                Ok(update) if !update.changed.is_empty() => emit_persist(
                    store,
                    session,
                    persist,
                    host,
                    json!({"event":"capability_map_bootstrapped","paths":update.changed}),
                )?,
                Ok(_) => {}
                Err(error) => emit_persist(
                    store,
                    session,
                    persist,
                    host,
                    json!({"event":"capability_map_warning","message":error.to_string()}),
                )?,
            }
        }
    }

    Ok(PreparedHarnessTurn {
        session_lock,
        role,
        active: active.clone(),
        explicit: explicit.cloned(),
        selected,
        prompt,
        semantic,
        image_selection,
        tools,
        turn_codegraph_binding,
        expected_codegraph_binding,
        ui_task,
    })
}
