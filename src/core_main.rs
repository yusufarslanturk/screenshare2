#[cfg(not(debug_assertions))]
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use crate::platform::breakdown_callback;
use hbb_common::log;
#[cfg(not(debug_assertions))]
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use hbb_common::platform::register_breakdown_handler;

#[cfg(any(target_os = "windows", target_os = "macos"))]
use std::fs;
	
use std::fs::write;
use hbb_common::{config::{Config},};

#[macro_export]
macro_rules! my_println{
    ($($arg:tt)*) => {
        #[cfg(not(windows))]
        println!("{}", format_args!($($arg)*));
        #[cfg(windows)]
        crate::platform::message_box(
            &format!("{}", format_args!($($arg)*))
        );
    };
}

#[inline]
fn is_empty_uni_link(arg: &str) -> bool {
    if !arg.starts_with("hoptodesk://") {
        return false;
    }
    arg["hoptodesk://".len()..].chars().all(|c| c == '/')
}

/// shared by flutter and sciter main function
///
/// [Note]
/// If it returns [`None`], then the process will terminate, and flutter gui will not be started.
/// If it returns [`Some`], then the process will continue, and flutter gui will be started.
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub fn core_main() -> Option<Vec<String>> {
    let mut args = Vec::new();
    let mut flutter_args = Vec::new();
    let mut i = 0;
    let mut _is_elevate = false;
    let mut _is_run_as_system = false;
    let mut _is_quick_support = false;
    let mut _is_flutter_invoke_new_connection = false;
    let mut arg_exe = Default::default();
    for arg in std::env::args() {
        if i == 0 {
            arg_exe = arg;
        } else if i > 0 {
            #[cfg(feature = "flutter")]
            if [
                "--connect",
                "--play",
                "--file-transfer",
                "--port-forward",
                "--rdp",
            ]
            .contains(&arg.as_str())
            {
                _is_flutter_invoke_new_connection = true;
            }
            if arg == "--elevate" {
                _is_elevate = true;
            } else if arg == "--run-as-system" {
                _is_run_as_system = true;
            } else if arg == "--quick_support" {
                _is_quick_support = true;
            } else {
                args.push(arg);
            }
        }
        i += 1;
    }
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    if args.is_empty() {
        if crate::check_process("--server", false) && !crate::check_process("--tray", true) {
            #[cfg(target_os = "linux")]
            hbb_common::allow_err!(crate::platform::check_autostart_config());
            //hbb_common::allow_err!(crate::run_me(vec!["--tray"]));
        }
    }
    #[cfg(not(debug_assertions))]
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    register_breakdown_handler(breakdown_callback);
    #[cfg(target_os = "linux")]
    #[cfg(feature = "flutter")]
    {
        let (k, v) = ("LIBGL_ALWAYS_SOFTWARE", "1");
        if !hbb_common::config::Config::get_option("allow-always-software-render").is_empty() {
            std::env::set_var(k, v);
        } else {
            std::env::remove_var(k);
        }
    }
    #[cfg(feature = "flutter")]
    if _is_flutter_invoke_new_connection {
        return core_main_invoke_new_connection(std::env::args());
    }
    let click_setup = cfg!(windows) && args.is_empty() && crate::common::is_setup(&arg_exe);
    if click_setup {
        args.push("--install".to_owned());
        flutter_args.push("--install".to_string());
    }
    if args.contains(&"--noinstall".to_string()) {
        args.clear();
    }
    if args.len() > 0 && args[0] == "--version" {
        println!("{}", crate::VERSION);
        return None;
    }
    let mut log_name = "".to_owned();
    if args.len() > 0 && args[0].starts_with("--") {
        let name = args[0].replace("--", "");
        if !name.is_empty() {
            log_name = name;
        }
    }
    hbb_common::init_log(false, &log_name);

    // linux uni (url) go here.
    #[cfg(all(target_os = "linux", feature = "flutter"))]
    if args.len() > 0 && args[0].starts_with("hoptodesk:") {
        return try_send_by_dbus(args[0].clone());
    }

    #[cfg(windows)]
    if !crate::platform::is_installed()
        && args.is_empty()
        && _is_quick_support
        && !_is_elevate
        && !_is_run_as_system
    {
        use crate::portable_service::client;
        if let Err(e) = client::start_portable_service(client::StartPara::Direct) {
            log::error!("Failed to start portable service:{:?}", e);
        }
    }
    #[cfg(windows)]
    if !crate::platform::is_installed() && (_is_elevate || _is_run_as_system) {
        crate::platform::elevate_or_run_as_system(click_setup, _is_elevate, _is_run_as_system);
        return None;
    }
    #[cfg(all(feature = "flutter", feature = "plugin_framework"))]
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    init_plugins(&args);
    log::info!("main start args:{:?}", args);
    if args.is_empty() || is_empty_uni_link(&args[0]) {
        std::thread::spawn(move || crate::start_server(false));
    } else {
        #[cfg(windows)]
        {
            use crate::platform;
            if args[0] == "--uninstall" {
                if let Err(err) = platform::uninstall_me(true) {
                    log::error!("Failed to uninstall: {}", err);
                }
                return None;
            } else if args[0] == "--after-install" {
                if let Err(err) = platform::run_after_install() {
                    log::error!("Failed to after-install: {}", err);
                }
                return None;
            } else if args[0] == "--before-uninstall" {
                if let Err(err) = platform::run_before_uninstall() {
                    log::error!("Failed to before-uninstall: {}", err);
                }
                return None;
            } else if args[0] == "--silent-install" {
				hbb_common::allow_err!(platform::install_me(
                    "desktopicon startmenu",
                    "".to_owned(),
                    true,
                    args.len() > 1,
					false
                ));
                return None;
            } else if args[0] == "--silent-install-noshortcuts" {
                hbb_common::allow_err!(platform::install_me(
                    "",
                    "".to_owned(),
                    true,
                    args.len() > 2 || (args.len() > 1 && args.get(1) != Some(&"--nostartup".to_owned())),
                    args.get(1) == Some(&"--nostartup".to_owned())
                ));
                return None;						
            } else if args[0] == "--extract" {
                #[cfg(feature = "with_rc")]
                hbb_common::allow_err!(crate::rc::extract_resources(&args[1]));
                return None;
            } else if args[0] == "--tray" {
                crate::tray::start_tray();
                return None;
            } else if args[0] == "--portable-service" {
                crate::platform::elevate_or_run_as_system(
                    click_setup,
                    _is_elevate,
                    _is_run_as_system,
                );
                return None;
            }
        }
		if args[0] == "--connect" {
			let input = &args[1];
				
			if input != "hoptodesk:///" {

				if input.starts_with("hoptodesk://connect/") {

					let id_with_password = input.strip_prefix("hoptodesk://connect/").unwrap();
					let mut new_args = args.clone();  // Create a new vector to modify

					let mut parts = id_with_password.splitn(4, '/');
					if let Some(id) = parts.next().map(str::to_owned) {
						new_args[1] = id.to_string();
					}
					if let Some(password) = parts.next() {
						new_args.push(password.to_owned());
						
					}


					let config_path = Config::path("TeamID.toml");
					
					#[cfg(any(target_os = "windows", target_os = "macos"))]
					if let Some(parent_dir) = config_path.parent() {
						if !parent_dir.exists() {
							fs::create_dir_all(parent_dir)
								.expect("Failed to create directory for TeamID.toml");
						}
					}
					if let Some(teamid) = parts.next() {
						if teamid.len() == 16 {
							write(&Config::path("TeamID.toml"), teamid).expect("Failed to write TeamID to file");
							//Config::set_option("teamidx".to_owned(), teamid.to_string());
						}
					}					
					if let Some(tokenex) = parts.next() {
						write(&Config::path("LastToken.toml"), tokenex).expect("Failed to write tokenex to file");
					}					
					
					args = new_args;  // Assign the modified vector back to args
				} else if input.starts_with("hoptodesk://filetransfer/") {
					if let Some(id) = input.strip_prefix("hoptodesk://filetransfer/").map(str::to_owned) {
						args[1] = id.to_string();
						args[0] = "--file-transfer".to_string();
					}
				} else if input.starts_with("hoptodesk://sync/") {
					if let Some(id) = input.strip_prefix("hoptodesk://sync/").map(str::to_owned) {
						args[1] = id.to_string();
						if args[1].is_empty() {
							hbb_common::config::Config::set_option("custom-api-url".to_owned(), "".to_owned());
						} else {
							hbb_common::config::Config::set_option("custom-api-url".to_owned(), format!("https://api.hoptodesk.com/?n={}", args[1]));
						}
						std::process::exit(0);
					}
				}
			}
		}
		if args[0] == "--remove" {
            if args.len() == 2 {
                // sleep a while so that process of removed exe exit
                std::thread::sleep(std::time::Duration::from_secs(1));
                std::fs::remove_file(&args[1]).ok();
                return None;
            }
        } else if args[0] == "--tray" {
            if !crate::check_process("--tray", true) {
                crate::tray::start_tray();
				std::process::exit(0);
            }
            return None;
        } else if args[0] == "--update" {
			let mut lastpath = "".to_string();
			let exe_path = std::env::current_exe().expect("Failed to get current executable path");
			#[cfg(windows)]
			if fs::metadata(Config::path("UpdatePath.toml")).is_ok() {
				lastpath = std::fs::read_to_string(Config::path("UpdatePath.toml")).unwrap_or_else(|err| {
						log::error!(
							"Error reading file: {:?}({})",
							Config::path("UpdatePath.toml").to_str(),
							err
						);
						String::new()
					});

				if crate::platform::is_installed() {
					let (_subkey, mut path, _start_menu, _, _) = crate::platform::windows::get_install_info();
					path.push_str("\\HopToDesk.exe");
					let _ = crate::platform::windows::run_uac_hide("sc", "stop HopToDesk");
					let _ = crate::platform::windows::run_uac_hide("taskkill", &format!("/F /IM {:?}.exe", &"HopToDesk"));
					std::thread::sleep(std::time::Duration::from_secs(10));
					let _ = fs::remove_file(path.clone());
					let _ = fs::copy(&exe_path, &path);
					std::thread::sleep(std::time::Duration::from_secs(1));
					let _ = crate::platform::windows::run_uac_hide("sc", "start HopToDesk");
				}			

				if let Err(err) = crate::platform::windows::run_uac_hide("taskkill", &format!("/F /IM {:?}.exe", &"HopToDesk")) {
					
				} else {
					let _ = fs::remove_file(lastpath.clone());
					let _ = fs::copy(&exe_path, &lastpath.clone());
				}

				if crate::platform::is_installed() {
					let _ = crate::platform::windows::run_uac_hide("sc", "start HopToDesk");
				}

				use std::process::Command;
				std::thread::sleep(std::time::Duration::from_secs(5));
				let status = Command::new(lastpath.clone()).status().expect("Failed to execute command");
			}

            std::process::exit(0);
        } else if args[0] == "--install-service" {
            log::info!("start --install-service");
            crate::platform::install_service();
            return None;
        } else if args[0] == "--uninstall-service" {
            log::info!("start --uninstall-service");
            crate::platform::uninstall_service(false);
        } else if args[0] == "--service" {
            #[cfg(target_os = "macos")]
            crate::platform::macos::hide_dock();
            log::info!("start --service");
            crate::start_os_service();
            return None;
        } else if args[0] == "--server" {
            log::info!("start --server with user {}", crate::username());
            #[cfg(all(windows, feature = "virtual_display_driver"))]
            crate::privacy_mode::restore_reg_connectivity();
            #[cfg(any(target_os = "linux", target_os = "windows"))]
            {
                crate::start_server(true);
                return None;
            }
            #[cfg(target_os = "macos")]
            {
                let handler = std::thread::spawn(move || crate::start_server(true));
                crate::tray::start_tray();
                // prevent server exit when encountering errors from tray
                hbb_common::allow_err!(handler.join());
            }
        } else if args[0] == "--import-config" {
			if args.len() == 2 {
                let filepath;
                let path = std::path::Path::new(&args[1]);
                if !path.is_absolute() {
					let mut cur = std::env::current_dir().unwrap();
                    cur.push(path);
                    filepath = cur.to_str().unwrap().to_string();
                } else {
					filepath = path.to_str().unwrap().to_string();
                }
				import_config(&filepath);
            }
            return None;
        } else if args[0] == "--password" {
            if args.len() == 2 {
                if crate::platform::is_root() {
                    crate::ipc::set_permanent_password(args[1].to_owned()).unwrap();
                } else {
                    println!("Administrative privileges required!");
                }
            }
            return None;
        } else if args[0] == "--get-id" {
			println!("{}", crate::ipc::get_id());
			//return None;
			std::process::exit(0);
/*			if crate::platform::is_root() {
                println!("{}", crate::ipc::get_id());
            } else {
				println!("Permission denied!");
            }			
            return None;*/
        } else if args[0] == "--check-hwcodec-config" {
            #[cfg(feature = "hwcodec")]
            scrap::hwcodec::check_config();
            return None;
        } else if args[0] == "--cm" {
            // call connection manager to establish connections
            // meanwhile, return true to call flutter window to show control panel
            crate::ui_interface::start_option_status_sync();
        } else if args[0] == "--cm-no-ui" {
            #[cfg(feature = "flutter")]
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            crate::flutter::connection_manager::start_cm_no_ui();
            return None;
        } else {
            #[cfg(all(feature = "flutter", feature = "plugin_framework"))]
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            if args[0] == "--plugin-install" {
                if args.len() == 2 {
                    crate::plugin::change_uninstall_plugin(&args[1], false);
                } else if args.len() == 3 {
                    crate::plugin::install_plugin_with_url(&args[1], &args[2]);
                }
                return None;
            } else if args[0] == "--plugin-uninstall" {
                if args.len() == 2 {
                    crate::plugin::change_uninstall_plugin(&args[1], true);
                }
                return None;
            }
        }
    }
    //_async_logger_holder.map(|x| x.flush());
    #[cfg(feature = "flutter")]
    return Some(flutter_args);
    #[cfg(not(feature = "flutter"))]
    return Some(args);
}

#[inline]
#[cfg(all(feature = "flutter", feature = "plugin_framework"))]
#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn init_plugins(args: &Vec<String>) {
    if args.is_empty() || "--server" == (&args[0] as &str) {
        #[cfg(debug_assertions)]
        let load_plugins = true;
        #[cfg(not(debug_assertions))]
        let load_plugins = crate::platform::is_installed();
        if load_plugins {
            crate::plugin::init();
        }
    } else if "--service" == (&args[0] as &str) {
        hbb_common::allow_err!(crate::plugin::remove_uninstalled());
    }
}

fn import_config(path: &str) {
    use hbb_common::{config::*, get_exe_time, get_modified_time};
    let path2 = path.replace(".toml", "2.toml");
    let path2 = std::path::Path::new(&path2);
    let path = std::path::Path::new(path);
    log::info!("import config from {:?} and {:?}", path, path2);
    let config: Config = load_path(path.into());
    if config.is_empty() {
        log::info!("Empty source config, skipped");
        return;
    }
    if get_modified_time(&path) > get_modified_time(&Config::file())
        && get_modified_time(&path) < get_exe_time()
    {
        if store_path(Config::file(), config).is_err() {
            log::info!("config written");
        }
    }
    let config2: Config2 = load_path(path2.into());
    if get_modified_time(&path2) > get_modified_time(&Config2::file()) {
        if store_path(Config2::file(), config2).is_err() {
            log::info!("config2 written");
        }
    }
}

/// invoke a new connection
///
/// [Note]
/// this is for invoke new connection from dbus.
/// If it returns [`None`], then the process will terminate, and flutter gui will not be started.
/// If it returns [`Some`], then the process will continue, and flutter gui will be started.
#[cfg(feature = "flutter")]
fn core_main_invoke_new_connection(mut args: std::env::Args) -> Option<Vec<String>> {
    args.position(|element| {
        return element == "--connect" || element == "--play";
    })?;
    let mut peer_id = args.next().unwrap_or("".to_string());
    if peer_id.is_empty() {
        eprintln!("please provide a valid peer id");
        return None;
    }
    let app_name = crate::get_app_name();
    let ext = format!(".{}", app_name.to_lowercase());
    if peer_id.ends_with(&ext) {
        peer_id = peer_id.replace(&ext, "");
    }
    let mut switch_uuid = None;
    while let Some(item) = args.next() {
        if item == "--switch_uuid" {
            switch_uuid = args.next();
        }
    }
    let mut param_array = vec![];
    if switch_uuid.is_some() {
        let switch_uuid = switch_uuid.map_or("".to_string(), |p| format!("switch_uuid={}", p));
        param_array.push(switch_uuid);
    }

    let params = param_array.join("&");
    let params_flag = if params.is_empty() { "" } else { "?" };
    #[allow(unused)]
    let uni_links = format!(
        "hoptodesk://connection/new/{}{}{}",
        peer_id, params_flag, params
    );

    #[cfg(target_os = "linux")]
    return try_send_by_dbus(uni_links);

    #[cfg(windows)]
    {
        use winapi::um::winuser::WM_USER;
        let res = crate::platform::send_message_to_hnwd(
            "FLUTTER_RUNNER_WIN32_WINDOW",
            "HopToDesk",
            (WM_USER + 2) as _, // referred from unilinks desktop pub
            uni_links.as_str(),
            false,
        );
        return if res { None } else { Some(Vec::new()) };
    }
    #[cfg(target_os = "macos")]
    {
        return if let Err(_) = crate::ipc::send_url_scheme(uni_links) {
            Some(Vec::new())
        } else {
            None
        };
    }
}

#[cfg(all(target_os = "linux", feature = "flutter"))]
fn try_send_by_dbus(uni_links: String) -> Option<Vec<String>> {
    use crate::dbus::invoke_new_connection;

    match invoke_new_connection(uni_links) {
        Ok(()) => {
            return None;
        }
        Err(err) => {
            log::error!("{}", err.as_ref());
            // return Some to invoke this url by self
            return Some(Vec::new());
        }
    }
}
