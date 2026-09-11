use crate::{AgentError, AgentResult, get_public_skill, get_public_skill_resource};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillUnit {
    pub id: String,
    pub hash: String,
    pub dependencies: Vec<String>,
    pub resource: String,
    pub content: String,
}

struct Unit {
    id: &'static str,
    bundle: &'static str,
    resource: &'static str,
    sections: &'static [&'static str],
}

const UNITS: &[Unit] = &[
    Unit {
        id: "core",
        bundle: "core",
        resource: "references/main.md",
        sections: &["Root files", "Long declarations", "Type declarations"],
    },
    Unit {
        id: "core/configuration",
        bundle: "core",
        resource: "references/main.md",
        sections: &["Root files"],
    },
    Unit {
        id: "core/validation",
        bundle: "core",
        resource: "references/workflow.md",
        sections: &[],
    },
    Unit {
        id: "theme",
        bundle: "theme",
        resource: "references/theme.md",
        sections: &[],
    },
    Unit {
        id: "views/layouts",
        bundle: "views",
        resource: "references/views.md",
        sections: &[
            "Named declarations",
            "Routes",
            "Layouts and pages",
            "Container decisions",
        ],
    },
    Unit {
        id: "views/pages",
        bundle: "views",
        resource: "references/views.md",
        sections: &[
            "Named declarations",
            "Layouts and pages",
            "Repeated views",
            "State",
        ],
    },
    Unit {
        id: "views/components",
        bundle: "views",
        resource: "references/composition.md",
        sections: &["Reusable components", "Repeated collection ownership"],
    },
    Unit {
        id: "views/svg",
        bundle: "views",
        resource: "references/svg.md",
        sections: &[],
    },
    Unit {
        id: "views/requests",
        bundle: "views",
        resource: "references/views.md",
        sections: &["View-to-server contracts", "View function utilities"],
    },
    Unit {
        id: "server/entities",
        bundle: "server",
        resource: "references/data.md",
        sections: &["Entities and seeders", "Relations"],
    },
    Unit {
        id: "server/handlers",
        bundle: "server",
        resource: "references/server.md",
        sections: &[
            "Named server declarations",
            "Capability-first statement shape",
            "View request consumers",
        ],
    },
    Unit {
        id: "server/functions",
        bundle: "server",
        resource: "references/server.md",
        sections: &[
            "Named server declarations",
            "Capability-first statement shape",
            "General function utilities",
        ],
    },
    Unit {
        id: "server/routes",
        bundle: "server",
        resource: "references/server.md",
        sections: &["Routes in `main.dowe`", "Endpoint routing"],
    },
    Unit {
        id: "server/persistence",
        bundle: "server",
        resource: "references/data.md",
        sections: &[
            "Handles",
            "Database queries",
            "Cache KV operations",
            "Vector embedding operations",
        ],
    },
    Unit {
        id: "domain-modeling",
        bundle: "domain-modeling",
        resource: "references/workflow.md",
        sections: &[],
    },
    Unit {
        id: "native-ipc",
        bundle: "native-ipc",
        resource: "references/ipc.md",
        sections: &[],
    },
];

