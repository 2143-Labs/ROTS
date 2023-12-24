use quote::{format_ident, quote};
use regex::Regex;
use std::{env, fs, path::Path};

fn generate_code(req: &GenerateRequest) -> String {
    let contents = std::fs::read_to_string(req.source).unwrap();

    // loop through the source file and look for all struct definitions.
    let all_types: Vec<_> = req
        .struct_search_regex
        .captures_iter(&contents)
        .map(|x| format_ident!("{}", &x[1]))
        .collect();

    let all_types_lowercase: Vec<_> = all_types
        .iter()
        .map(|x| format_ident!("writer_{}", x.to_string().to_lowercase()))
        .collect();

    let typename = format_ident!("EventTo{}", req.output_type_name);

    let code = quote!(
        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[non_exhaustive]
        pub enum #typename {
            #( #all_types ( #all_types ) ),*
        }

        pub fn drain_events (
            sr: Res<ServerResources<#typename>>,
            #(
                mut #all_types_lowercase: EventWriter<EventFromEndpoint< #all_types >>
            ),*
        ) {
            let mut new_events = sr.event_list.lock().unwrap();
            let new_events = std::mem::replace(new_events.as_mut(), vec![]);
            for (endpoint, event) in new_events {
                match event {
                    #(
                        #typename :: #all_types (data) => {
                            #all_types_lowercase . send(EventFromEndpoint::new(endpoint, data));
                        }
                    ),*
                }
            }
        }

        pub fn register_events(app: &mut App) {
            #(
                app.add_event::< EventFromEndpoint< #all_types > >();
            )*
        }
    );

    let code = syn::parse_file(&code.to_string()).unwrap();

    prettyplease::unparse(&code)
    //code.to_string()
}

fn generate_systems(req: GenerateRequest) {
    let code_str = generate_code(&req);

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join(req.output_filename);
    fs::write(dest_path, code_str).unwrap();
}

struct GenerateRequest<'a> {
    output_filename: &'a str,
    output_type_name: &'a str,
    source: &'a str,
    struct_search_regex: &'a Regex,
}

fn main() {
    let r = Regex::new(r#"(?:struct|enum) (\w+?) \{"#).unwrap();
    generate_systems(GenerateRequest {
        source: "src/event/client.rs",
        output_filename: "./client_event.rs",
        output_type_name: "Client",
        struct_search_regex: &r,
    });

    generate_systems(GenerateRequest {
        source: "src/event/server.rs",
        output_filename: "./server_event.rs",
        output_type_name: "Server",
        struct_search_regex: &r,
    });

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/event/server.rs");
    println!("cargo:rerun-if-changed=src/event/client.rs");
}
