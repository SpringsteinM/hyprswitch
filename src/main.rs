use std::error;

use hyprland::data::{Animations, Client, Clients, Monitors, Workspace, Workspaces};
use hyprland::event_listener::AsyncEventListener;
use hyprland::keyword::*;
use hyprland::prelude::*;
//use hyprland::shared::WorkspaceType;
use hyprland::{async_closure, dispatch::*};
use serde::{Deserialize, Serialize};

use clap::Parser;

#[derive(Debug, Deserialize, Serialize)]
struct State {
    group_previous_workspace: i32,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
struct Args {
    #[arg(short, long)]
    application: Option<String>,

    #[arg(short, long)]
    workspace: i32,

    #[arg(short, long, num_args = 0.., value_delimiter = ' ')]
    group: Vec<i32>,
}

fn load_state() -> Result<State, Box<dyn std::error::Error>> {
    let home_dir = std::env::var_os("HOME").ok_or("no home directory")?;
    let mut state_path = std::path::PathBuf::new();
    state_path.push(home_dir);
    state_path.push(".local/share/hyprswitch/state.toml");
    let config_content = std::fs::read_to_string(&state_path)?;
    let config: State = toml::from_str(&config_content).unwrap();
    Ok(config)
}

fn save_state(state: &State) -> Result<(), Box<dyn std::error::Error>> {
    let home_dir = std::env::var_os("HOME").ok_or("no home directory")?;
    let mut state_path = std::path::PathBuf::new();
    state_path.push(home_dir);
    state_path.push(".local/share/hyprswitch/state.toml");

    let dir_name = state_path.parent().ok_or("incorrect directory")?;
    std::fs::create_dir_all(dir_name)?;
    let content = toml::to_string(state).unwrap();
    std::fs::write(&state_path, content)?;
    Ok(())
}

#[tokio::main]
async fn main() -> hyprland::Result<()> {
    let args = Args::parse();

    let mut state = match load_state() {
        Ok(s) => s,
        Err(_) => State {
            group_previous_workspace: (-1),
        },
    };

    println!("{state:#?}");
    let work = Workspace::get_active_async().await?;

    // Save current workspace in state file
    if !args.group.contains(&work.id) && args.group.contains(&args.workspace) {
        state.group_previous_workspace = work.id;
        save_state(&state).unwrap();
    }

    if work.id == args.workspace {
        if args.group.contains(&work.id) && state.group_previous_workspace > 0 {
            Dispatch::call_async(DispatchType::Workspace(WorkspaceIdentifierWithSpecial::Id(
                state.group_previous_workspace,
            )))
            .await?;
        } else {
            Dispatch::call_async(DispatchType::Workspace(
                WorkspaceIdentifierWithSpecial::Previous,
            ))
            .await?;
        }
    } else {
        Dispatch::call_async(DispatchType::Workspace(WorkspaceIdentifierWithSpecial::Id(
            args.workspace,
        )))
        .await?;
    }

    // let win = Client::get_active_async().await?;
    // // and all open windows
    if args.application.is_none() {
        return Ok(());
    }

    let command = args.application.unwrap_or("".to_owned());
    let new_class_name = command.split_whitespace().next().unwrap_or("");

    let clients = Clients::get_async().await?;
    let mut found = false;
    for client in clients {
        if client.workspace.id == args.workspace {
            let class_name = client.class.to_lowercase();

            if class_name == new_class_name {
                found = true;
                println!("{client:#?}");
                // println!("{class_name:#?}");
            }
        }
    }

    if !found {
        hyprland::dispatch!(async; Exec, &command).await?;
    }

    // // and the active workspace
    // let work = Workspace::get_active_async().await?;
    // // and printing them all out!
    // println!("monitors: {monitors:#?},\nactive window: {win:#?},\nclients {clients:#?}\nworkspace:{work:#?}");
    // let animations = Animations::get_async().await?;
    // println!("{animations:#?}");
    // // Create a event listener
    // let mut event_listener = AsyncEventListener::new();

    // //This changes the workspace to 5 if the workspace is switched to 9
    // //this is a performance and mutable state test
    // // event_listener.add_workspace_change_handler(async_closure! {|id, state| {
    // //     if id == WorkspaceType::Regular('9'.to_string()) {
    // //         *state.workspace = '2'.to_string();
    // //     }
    // // }});
    // /*
    // event_listener.add_workspace_change_handler(|id, state| {
    //     Box::pin(async move {
    //         if id == WorkspaceType::Regular('9'.to_string()) {
    //             *state.workspace = '2'.to_string();
    //         }
    //     })
    // });

    // // This makes it so you can't turn on fullscreen lol
    // event_listener.add_fullscreen_state_change_handler(async_closure! {|fstate, state| {
    //     if fstate {
    //         *state.fullscreen = false;
    //     }
    // }});
    // // Makes a monitor unfocusable
    // event_listener.add_active_monitor_change_handler(async_closure! {|data, state| {
    //     let hyprland::event_listener::MonitorEventData{ monitor_name, .. } = data;

    //     if monitor_name == *"DP-1".to_string() {
    //         *state.monitor = "eDP-1".to_string()
    //     }
    // }});
    // */
    // // add event, yes functions and closures both work!

    // event_listener.add_workspace_change_handler(
    //     async_closure! { move |id| println!("workspace changed to {id:#?}")},
    // );
    // event_listener.add_active_window_change_handler(
    //     async_closure! { move |data| println!("window changed to {data:#?}")},
    // );
    // // Waybar example
    // // event_listener.add_active_window_change_handler(|data| {
    // //     use hyprland::event_listener::WindowEventData;
    // //     let string = match data {
    // //         Some(WindowEventData(class, title)) => format!("{class}: {title}"),
    // //         None => "".to_string()
    // //     };
    // //     println!(r#"{{"text": "{string}", class: "what is this?"}}"#);
    // // });

    // // and execute the function
    // // here we are using the blocking variant
    // // but there is a async version too
    // event_listener.start_listener_async().await
    Ok(())
}
