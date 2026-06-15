use tauri::{
    AppHandle, Emitter, Manager,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
use tauri::menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder};

pub fn create_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let menu = build_menu(app)?;
    let mut tray_builder = TrayIconBuilder::new()
        .tooltip("SelfHost Helper")
        .menu(&menu);
    if let Some(icon) = app.default_window_icon().cloned() {
        tray_builder = tray_builder.icon(icon);
    }
    let _tray = tray_builder
        .on_menu_event(move |app, event| {
            handle_menu_event(app, &event.id().0);
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;
    Ok(())
}

fn build_menu(app: &AppHandle) -> Result<tauri::menu::Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    let show = MenuItemBuilder::with_id("show", "Show Window").build(app)?;
    let hide = MenuItemBuilder::with_id("hide", "Hide Window").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
    let separator = PredefinedMenuItem::separator(app)?;

    let projects_menu = SubmenuBuilder::new(app, "Projects")
        .item(&MenuItemBuilder::with_id("project_add", "Add Project").build(app)?)
        .item(&MenuItemBuilder::with_id("project_open", "Open Selected").build(app)?)
        .item(
            &MenuItemBuilder::with_id("project_open_dir", "Open Directory").build(app)?,
        )
        .separator()
        .item(&MenuItemBuilder::with_id("project_stop", "Stop Process").build(app)?)
        .item(
            &MenuItemBuilder::with_id("project_start", "Start Process").build(app)?,
        )
        .item(
            &MenuItemBuilder::with_id("project_restart", "Restart Process").build(app)?,
        )
        .build()?;

    let tools_menu = SubmenuBuilder::new(app, "Tools")
        .item(
            &MenuItemBuilder::with_id("tunnel_stop_all", "Stop All Tunnels").build(app)?,
        )
        .item(
            &MenuItemBuilder::with_id("check_updates", "Check for Updates").build(app)?,
        )
        .build()?;

    let menu = MenuBuilder::new(app)
        .item(&show)
        .item(&hide)
        .item(&separator)
        .item(&projects_menu)
        .item(&tools_menu)
        .item(&PredefinedMenuItem::separator(app)?)
        .item(&quit)
        .build()?;

    Ok(menu)
}

fn handle_menu_event(app: &AppHandle, id: &str) {
    match id {
        "show" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        "hide" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.hide();
            }
        }
        "quit" => {
            app.exit(0);
        }
        "project_add" => {
            let _ = app.emit("tray:project_add", ());
        }
        "project_open" => {
            let _ = app.emit("tray:project_open", ());
        }
        "project_open_dir" => {
            let _ = app.emit("tray:project_open_dir", ());
        }
        "project_stop" => {
            let _ = app.emit("tray:project_stop", ());
        }
        "project_start" => {
            let _ = app.emit("tray:project_start", ());
        }
        "project_restart" => {
            let _ = app.emit("tray:project_restart", ());
        }
        "tunnel_stop_all" => {
            let _ = app.emit("tray:tunnel_stop_all", ());
        }
        "check_updates" => {
            let _ = app.emit("tray:check_updates", ());
        }
        _ => {}
    }
}

pub async fn rebuild_tray_with_projects(
    app: &AppHandle,
    projects: Vec<serde_json::Value>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut projects_menu = SubmenuBuilder::new(app, "Projects");
    for project in &projects {
        let name = project
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("Unknown");
        let id = project
            .get("id")
            .and_then(|i| i.as_i64())
            .unwrap_or(0);
        projects_menu = projects_menu.item(
            &MenuItemBuilder::with_id(format!("tray_project_{}", id), name).build(app)?,
        );
    }
    projects_menu = projects_menu.separator();
    projects_menu = projects_menu
        .item(&MenuItemBuilder::with_id("project_add", "Add Project").build(app)?);

    let show = MenuItemBuilder::with_id("show", "Show Window").build(app)?;
    let hide = MenuItemBuilder::with_id("hide", "Hide Window").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
    let separator = PredefinedMenuItem::separator(app)?;

    let tools_menu = SubmenuBuilder::new(app, "Tools")
        .item(
            &MenuItemBuilder::with_id("tunnel_stop_all", "Stop All Tunnels").build(app)?,
        )
        .item(
            &MenuItemBuilder::with_id("check_updates", "Check for Updates").build(app)?,
        )
        .build()?;

    let menu = MenuBuilder::new(app)
        .item(&show)
        .item(&hide)
        .item(&separator)
        .item(&projects_menu.build()?)
        .item(&tools_menu)
        .item(&separator)
        .item(&quit)
        .build()?;

    if let Some(tray) = app.tray_by_id("main") {
        tray.set_menu(Some(menu))?;
    }
    Ok(())
}
