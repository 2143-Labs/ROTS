use std::env;
use std::fmt::Write;
use std::fs;
use std::path::Path;

use quote::format_ident;
use quote::quote;
use regex::Regex;

const CODE: &'static str = "
";

fn generate_code(req: &GenerateRequest) -> String {
    let contents = std::fs::read_to_string(req.source).unwrap();
    //let mut structbody = String::new();
    //let mut drain_events_systems = String::new();
    let all_types: Vec<_> = req
        .struct_search_regex
        .captures_iter(&contents)
        .map(|x| format_ident!("{}", &x[1]))
        .collect();

    let all_types_lowercase: Vec<_> = all_types.iter().map(|x| format_ident!("writer_{}", x)).collect();

    //let type_name =
    //writeln!(structbody, "{0}({0})", cap.as_str()).unwrap();
    //writeln!(drain_events_systems, "mut writer_{0}: EventWriter<{0}>,", cap.as_str()).unwrap();
    //}

    let typename = format_ident!("EventTo{}", req.output_type_name);
    let drain_name = format_ident!("drain_events_{}", req.output_type_name.to_lowercase());

    let code = quote!(
        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[non_exhaustive]
        pub enum #typename {
            #( #all_types ( #all_types ) ),*
        }

        pub fn #drain_name (
            sr: Res<ServerResources<#typename>>,
            #(
                mut #all_types_lowercase: EventWriter<#all_types>
            ),*
        ) {
            let mut new_events = sr.event_list.lock().unwrap();
            let new_events = std::mem::replace(new_events.as_mut(), vec![]);
            for (_endpoint, event) in new_events {
                match event {
                    #(
                        #typename :: #all_types (data) => {
                            #all_types_lowercase . send(data);
                        }
                    ),*
                }
            }
        }
    );

    let code = syn::parse_file(&code.to_string()).unwrap();
    let formatted = prettyplease::unparse(&code);
    formatted
}

fn generate_systems(req: GenerateRequest) {
    let code_str = generate_code(&req);

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join(req.output_filename);
    fs::write(&dest_path, &code_str).unwrap();
}

struct GenerateRequest<'a> {
    output_filename: &'a str,
    output_type_name: &'a str,
    source: &'a str,
    struct_search_regex: &'a Regex,
}

fn main() {
    let r = Regex::new(r#"struct (\w+?) \{"#).unwrap();
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
    println!("cargo:warning=TEST");
}
