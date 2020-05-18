
use log::{self,debug};
use exitfailure::ExitFailure;
use atty::Stream;
use structopt::StructOpt;
use fatcat_cli::{Specifier, FatcatApiClient};

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case", about = "CLI interface to Fatcat API" )]
struct Opt {

    #[structopt(long, env = "FATCAT_API_HOST", default_value = "https://api.fatcat.wiki")]
    api_host: String,

    #[structopt(long, env = "FATCAT_API_AUTH_TOKEN", hide_env_values = true)]
    api_token: Option<String>,

    //#[structopt(long, env = "FATCAT_SEARCH_HOST", default_value = "https://search.fatcat.wiki")]
    //search_host: String,

    /// Pass many times for more log output
    ///
    /// By default, it'll only report errors. Passing `-v` one time also prints
    /// warnings, `-vv` enables info logging, `-vvv` debug, and `-vvvv` trace.
    #[structopt(long, short = "v", parse(from_occurrences))]
    verbose: i8,

    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt)]
enum Command {
    Status,
    Get {
        specifier: Specifier,

        #[structopt(long)]
        toml: bool,
    },
    //Update {},
    //Create {},
    //Delete {},
    //Edit
    //Editgroup
    //Download
    //Search
}

fn main() -> Result<(), ExitFailure> {
    let opt = Opt::from_args();

    let log_level = match opt.verbose {
        std::i8::MIN..=-1 => "none",
        0 => "error",
        1 => "warn",
        2 => "info",
        3 => "debug",
        4..=std::i8::MAX => "trace",
    };
    env_logger::from_env(env_logger::Env::default().default_filter_or(log_level))
        .format_timestamp(None)
        .init();

    debug!("Args parsed, starting up");

    if atty::is(Stream::Stdout) {
        //println!("I'm a terminal");
    } else {
        //println!("I'm not");
    }

    let client = if opt.api_host.starts_with("https://") {
        // Using Simple HTTPS
        fatcat_openapi::client::Client::try_new_https(&opt.api_host).expect("Failed to create HTTPS client")
    } else if opt.api_host.starts_with("http://") {
        // Using HTTP
        fatcat_openapi::client::Client::try_new_http(&opt.api_host).expect("Failed to create HTTP client")
    } else {
        panic!("unsupported API Host prefix");
    };

    let api_client = FatcatApiClient::new(&client);

    match opt.cmd {
        Command::Get {toml, specifier} => {
            let result = specifier.get_from_api(api_client)?;
            if toml {
                println!("{}", result.to_toml_string()?)
            } else {
                println!("{}", result.to_json_string()?)
            }
        },
        Command::Status => {
            println!("All good!");
        },
    }
    Ok(())
}
