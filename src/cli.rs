//! CLI hierarchy and argument definitions using Clap derive.

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "tic",
    version,
    about = "Native CLI for the TIC search and datasets API (company, person, vehicle, and bankruptcy lookup)",
    after_help = "An LLM agent should start with: tic agent-readme",
    propagate_version = true,
    disable_help_subcommand = true
)]
pub struct Cli {
    /// Print raw JSON instead of YAML, for scripting
    #[arg(long, global = true)]
    pub json: bool,

    /// Output format (yaml, json)
    #[arg(long, global = true, value_name = "FORMAT")]
    pub format: Option<String>,

    /// Account to run against (see 'tic accounts list')
    #[arg(short = 'a', long, global = true, value_name = "ACCOUNT")]
    pub account: Option<String>,

    /// TIC API key (overrides stored account and TIC_API_KEY env var)
    #[arg(long, global = true, value_name = "KEY")]
    pub api_key: Option<String>,

    /// Print HTTP method, URL, and status code to stderr
    #[arg(long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Company search and dataset lookup
    #[command(subcommand)]
    Company(CompanyCommand),

    /// Swedish bankruptcy records lookup
    #[command(subcommand)]
    Bankruptcy(BankruptcyCommand),

    /// Person search and lookup
    #[command(subcommand)]
    Person(PersonCommand),

    /// Swedish vehicle registry lookup
    #[command(subcommand)]
    Vehicle(VehicleCommand),

    /// Raw search across any TIC collection
    Search(SearchArgs),

    /// Log in with a TIC API key (stores key securely in OS keystore)
    Login(LoginArgs),

    /// Manage configured TIC accounts and keys
    #[command(subcommand)]
    Accounts(AccountsCommand),

    /// Print the operating manual for an LLM agent driving this CLI
    AgentReadme,
}

// -----------------------------------------------------------------------------
// Company commands
// -----------------------------------------------------------------------------

#[derive(Subcommand)]
pub enum CompanyCommand {
    /// Look up company by registration number (e.g. 556792-6687)
    Get {
        /// Organization / registration number
        #[arg(value_name = "REG_NR")]
        reg_nr: String,
    },

    /// Look up company by internal TIC company ID
    GetById {
        /// Internal TIC company identifier
        #[arg(value_name = "COMPANY_ID")]
        company_id: u64,
    },

    /// Search companies by name, address, phone, email, or other fields
    Search {
        /// Search query (use * for wildcard)
        #[arg(value_name = "QUERY")]
        query: String,

        /// Fields to search (comma-separated, default: names.nameOrIdentifier)
        #[arg(long, default_value = "names.nameOrIdentifier")]
        query_by: String,

        /// Filter expression (e.g. isCeased:false)
        #[arg(long)]
        filter_by: Option<String>,

        /// Sort expression (e.g. registrationDate:desc)
        #[arg(long)]
        sort_by: Option<String>,

        /// Results per page (default: 10)
        #[arg(long, default_value_t = 10)]
        per_page: usize,
    },

    /// Get ownership graph for a company
    Graph {
        /// Internal TIC company identifier
        #[arg(value_name = "COMPANY_ID")]
        company_id: u64,

        /// Include beneficial owners in graph
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        include_beneficial_owners: bool,

        /// Maximum total nodes in graph
        #[arg(long, default_value_t = 10000)]
        max_nodes: usize,

        /// Exclude persons with more than N roles
        #[arg(long)]
        max_roles: Option<usize>,
    },

    /// Get ownership tree for a company
    Tree {
        /// Internal TIC company identifier
        #[arg(value_name = "COMPANY_ID")]
        company_id: u64,
    },

    /// Get intelligence records (anomalies, warnings, red flags)
    Intelligence {
        /// Internal TIC company identifier
        #[arg(value_name = "COMPANY_ID")]
        company_id: u64,
    },

    /// Get beneficial owners for a company
    BeneficialOwners {
        /// Internal TIC company identifier
        #[arg(value_name = "COMPANY_ID")]
        company_id: u64,
    },

    /// Get board members, CEO, auditors, and other parties
    Parties {
        /// Internal TIC company identifier
        #[arg(value_name = "COMPANY_ID")]
        company_id: u64,
    },

    /// Get debtor summary (unpaid debts at Kronofogden)
    DebtorSummary {
        /// Internal TIC company identifier
        #[arg(value_name = "COMPANY_ID")]
        company_id: u64,
    },

    /// Get vehicles owned by a company
    Vehicles {
        /// Internal TIC company identifier
        #[arg(value_name = "COMPANY_ID")]
        company_id: u64,

        /// Page number (default: 1)
        #[arg(long, default_value_t = 1)]
        page: usize,

        /// Results per page (default: 100)
        #[arg(long, default_value_t = 100)]
        page_size: usize,
    },

    /// Get properties owned by a company
    Properties {
        /// Internal TIC company identifier
        #[arg(value_name = "COMPANY_ID")]
        company_id: u64,

        /// Include historical ownership (not just current)
        #[arg(long)]
        include_historical: bool,
    },
}

// -----------------------------------------------------------------------------
// Bankruptcy commands
// -----------------------------------------------------------------------------

#[derive(Subcommand)]
pub enum BankruptcyCommand {
    /// Search Swedish bankruptcy records
    Search {
        /// Search query (registration number, company name, or * for wildcard)
        #[arg(value_name = "QUERY")]
        query: String,

        /// Fields to search (comma-separated, default: registrationNumber)
        #[arg(long, default_value = "registrationNumber")]
        query_by: String,

        /// Filter expression (e.g. bankruptcyStatusCode:[20])
        #[arg(long)]
        filter_by: Option<String>,

        /// Sort expression (e.g. initiatedDate:desc)
        #[arg(long)]
        sort_by: Option<String>,

        /// Results per page (default: 10)
        #[arg(long, default_value_t = 10)]
        per_page: usize,
    },
}

// -----------------------------------------------------------------------------
// Person commands
// -----------------------------------------------------------------------------

#[derive(Subcommand)]
pub enum PersonCommand {
    /// Search persons by personal identity number (e.g. 198207174171, use % as wildcard)
    Search {
        /// Personal identity number
        #[arg(value_name = "QUERY")]
        query: String,

        /// Results per page (default: 100)
        #[arg(long, default_value_t = 100)]
        page_size: usize,
    },

    /// Get person details by TIC person ID
    Get {
        /// Internal TIC person identifier
        #[arg(value_name = "PERSON_ID")]
        person_id: u64,
    },

    /// Get all companies where a person has or had a role
    Companies {
        /// Internal TIC person identifier
        #[arg(value_name = "PERSON_ID")]
        person_id: u64,

        /// Results per page (default: 100)
        #[arg(long, default_value_t = 100)]
        page_size: usize,
    },
}

// -----------------------------------------------------------------------------
// Vehicle commands
// -----------------------------------------------------------------------------

#[derive(Subcommand)]
pub enum VehicleCommand {
    /// Search Swedish vehicle registry
    Search {
        /// Search query (licence plate, VIN, manufacturer, etc.)
        #[arg(value_name = "QUERY")]
        query: String,

        /// Fields to search (comma-separated, default: licencePlate)
        #[arg(long, default_value = "licencePlate")]
        query_by: String,

        /// Filter expression
        #[arg(long)]
        filter_by: Option<String>,

        /// Sort expression
        #[arg(long)]
        sort_by: Option<String>,

        /// Results per page (default: 10)
        #[arg(long, default_value_t = 10)]
        per_page: usize,
    },
}

// -----------------------------------------------------------------------------
// Raw search command
// -----------------------------------------------------------------------------

#[derive(Args, Clone)]
pub struct SearchArgs {
    /// Collection to search (e.g. companies, vehicles/se, companies/bankruptcies/se)
    #[arg(value_name = "COLLECTION")]
    pub collection: String,

    /// Search query (use * for wildcard)
    #[arg(value_name = "QUERY")]
    pub query: String,

    /// Fields to search (comma-separated)
    #[arg(long, default_value = "names.nameOrIdentifier")]
    pub query_by: String,

    /// Filter expression
    #[arg(long)]
    pub filter_by: Option<String>,

    /// Sort expression
    #[arg(long)]
    pub sort_by: Option<String>,

    /// Results per page (default: 10)
    #[arg(long, default_value_t = 10)]
    pub per_page: usize,
}

// -----------------------------------------------------------------------------
// Login & Accounts commands
// -----------------------------------------------------------------------------

#[derive(Args, Clone)]
pub struct LoginArgs {
    /// Account name to store (default: "default")
    #[arg(value_name = "NAME", default_value = "default")]
    pub name: String,

    /// TIC API key (prompted for securely if omitted)
    #[arg(long, value_name = "KEY", conflicts_with = "api_key_stdin")]
    pub api_key: Option<String>,

    /// Read the API key from stdin
    #[arg(long)]
    pub api_key_stdin: bool,

    /// Do not open the browser to the API keys page automatically
    #[arg(long)]
    pub no_browser: bool,

    /// Replace the key on an account that already exists
    #[arg(long)]
    pub force: bool,

    /// Store the key without verifying it first
    #[arg(long)]
    pub no_verify: bool,
}

#[derive(Subcommand)]
pub enum AccountsCommand {
    /// Add or update an account with an API key
    Add {
        /// Account name
        #[arg(value_name = "NAME")]
        name: String,

        /// TIC API key
        #[arg(long, value_name = "KEY", conflicts_with = "api_key_stdin")]
        api_key: Option<String>,

        /// Read the API key from stdin
        #[arg(long)]
        api_key_stdin: bool,

        /// Replace the key if the account already exists
        #[arg(long)]
        force: bool,

        /// Store without verifying against API
        #[arg(long)]
        no_verify: bool,
    },

    /// List configured accounts
    List {
        /// Test stored keys against the API
        #[arg(long)]
        check: bool,
    },

    /// Test stored key for an account
    Test {
        /// Account name
        #[arg(value_name = "NAME")]
        name: String,
    },

    /// Remove an account and its stored API key
    Remove {
        /// Account name
        #[arg(value_name = "NAME")]
        name: String,

        /// Skip confirmation prompt
        #[arg(long)]
        yes: bool,
    },
}
