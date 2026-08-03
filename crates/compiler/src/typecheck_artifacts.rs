include!("typecheck_artifacts/source_language.rs");

#[cfg(test)]
mod tests {
    use super::typecheck_artifacts;
    use std::path::Path;

    #[test]
    fn server_surface_describes_queue_connections_and_direct_publications() {
        let server = typecheck_artifacts()
            .into_iter()
            .find(|artifact| artifact.relative_path == Path::new("language/server-surface.json"))
            .expect("server surface");

        assert!(server.content.contains("queue appQueue provider"));
        assert!(server.content.contains("msg sent conn:appQueue.publish"));
        assert!(server.content.contains("msg.publish ok"));
        assert!(server.content.contains("msg.publish id"));
    }
}
