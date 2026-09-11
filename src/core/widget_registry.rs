pub const SERVICE_MONITOR_ID: &str = "service_monitor";
pub const CLOCK_ID: &str = "clock";
pub const SYSTEM_MONITOR_ID: &str = "system_monitor";

#[derive(Debug, Clone, Copy)]
pub struct WidgetDefinition {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub default_enabled: bool,
    pub default_x: f32,
    pub default_y: f32,
}

pub const WIDGET_DEFINITIONS: &[WidgetDefinition] = &[
    WidgetDefinition {
        id: SERVICE_MONITOR_ID,
        name: "Service Monitor",
        description: "Surveille les services HTTP/Downdetector et affiche leur état.",
        default_enabled: true,
        default_x: 30.0,
        default_y: 30.0,
    },
    WidgetDefinition {
        id: CLOCK_ID,
        name: "Horloge",
        description: "Affiche l'heure locale et, au choix, la date.",
        default_enabled: true,
        default_x: 390.0,
        default_y: 30.0,
    },
    WidgetDefinition {
        id: SYSTEM_MONITOR_ID,
        name: "Moniteur système",
        description: "CPU, GPU, RAM, capacité des disques, activité disque et uptime.",
        default_enabled: true,
        default_x: 390.0,
        default_y: 155.0,
    },
];

pub fn definition(id: &str) -> Option<&'static WidgetDefinition> {
    WIDGET_DEFINITIONS.iter().find(|widget| widget.id == id)
}
