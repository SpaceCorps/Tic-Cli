//! In-process TCP mock HTTP server and CLI integration tests for `tic`.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use serde_json::{Value, json};

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct Recorded {
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    body: Option<Value>,
}

type Route = (&'static str, &'static str, u16, Value);

struct Mock {
    url: String,
    log: Arc<Mutex<Vec<Recorded>>>,
}

impl Mock {
    fn start(routes: Vec<Route>) -> Mock {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/", listener.local_addr().unwrap());
        let log = Arc::new(Mutex::new(Vec::new()));
        let log2 = log.clone();
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let routes = routes.clone();
                let log = log2.clone();
                std::thread::spawn(move || {
                    let mut reader = BufReader::new(stream.try_clone().unwrap());
                    let mut line = String::new();
                    if reader.read_line(&mut line).unwrap_or(0) == 0 {
                        return;
                    }
                    let mut parts = line.split_whitespace();
                    let method = parts.next().unwrap_or("").to_string();
                    let full_path = parts.next().unwrap_or("").to_string();
                    let path = full_path.trim_start_matches('/').to_string();

                    let mut headers = Vec::new();
                    let mut len = 0usize;
                    loop {
                        let mut h = String::new();
                        reader.read_line(&mut h).unwrap();
                        let h = h.trim_end();
                        if h.is_empty() {
                            break;
                        }
                        if let Some((k, v)) = h.split_once(':') {
                            let (k, v) = (k.trim().to_lowercase(), v.trim().to_string());
                            if k == "content-length" {
                                len = v.parse().unwrap_or(0);
                            }
                            headers.push((k, v));
                        }
                    }
                    let mut buf = vec![0; len];
                    reader.read_exact(&mut buf).unwrap();
                    let body = (len > 0).then(|| serde_json::from_slice(&buf).unwrap_or(Value::Null));
                    log.lock().unwrap().push(Recorded { method: method.clone(), path: path.clone(), headers, body });

                    let (status, resp) = routes
                        .iter()
                        .find(|(m, p, _, _)| *m == method && *p == path)
                        .map(|(_, _, s, b)| (*s, b.clone()))
                        .unwrap_or((404, json!({"code": "not_found", "message": "no route matched"})));

                    let text = resp.to_string();
                    let _ = write!(
                        stream,
                        "HTTP/1.1 {status} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{text}",
                        text.len()
                    );
                });
            }
        });
        Mock { url, log }
    }

    #[allow(dead_code)]
    fn requests(&self) -> Vec<Recorded> {
        self.log.lock().unwrap().clone()
    }
}

struct Env {
    dir: PathBuf,
    api: String,
}

impl Env {
    fn new(mock: &Mock) -> Env {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "tic-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        Env { dir, api: mock.url.clone() }
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_tic"))
            .args(args)
            .env("TIC_CONFIG_DIR", &self.dir)
            .env("TIC_SECRET_STORE", "plaintext")
            .env("TIC_ALLOW_PLAINTEXT_STORE", "1")
            .env("TIC_API_URL", &self.api)
            .output()
            .unwrap()
    }

    fn json(&self, args: &[&str]) -> (i32, Value, Value) {
        let mut all = args.to_vec();
        all.push("--json");
        let out = self.run(&all);
        let parse = |b: &[u8]| {
            let s = String::from_utf8_lossy(b);
            let s = s.lines().filter(|l| !l.starts_with("warning:")).collect::<Vec<_>>().join("\n");
            serde_json::from_str(&s).unwrap_or(Value::Null)
        };
        (out.status.code().unwrap_or(-1), parse(&out.stdout), parse(&out.stderr))
    }

    fn with_account(self) -> Env {
        let (code, out, err) = self.json(&["accounts", "add", "work", "--api-key", "test_key", "--no-verify"]);
        assert_eq!(code, 0, "accounts add failed: {err}");
        assert_eq!(out["status"], "added");
        self
    }
}

impl Drop for Env {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------

#[test]
fn agent_readme_json() {
    let mock = Mock::start(vec![]);
    let env = Env::new(&mock);
    let (code, out, _) = env.json(&["agent-readme"]);
    assert_eq!(code, 0);
    assert_eq!(out["tool"], "tic");
    assert_eq!(out["apiVersion"], "1.0.0");
    assert!(out["rules"].is_array());
    assert_eq!(out["exitCodes"]["0"], "ok");
}

#[test]
fn invalid_arguments_envelope() {
    let mock = Mock::start(vec![]);
    let env = Env::new(&mock);
    let (code, _, err) = env.json(&["--nonexistent-flag"]);
    assert_eq!(code, 6);
    assert_eq!(err["code"], "invalid_input");
}

#[test]
fn accounts_lifecycle() {
    let mock = Mock::start(vec![("GET", "me", 200, json!({"email": "test@tic.io", "name": "Test User"}))]);
    let env = Env::new(&mock);

    // 1. Add account with verify
    let (code, out, err) = env.json(&["accounts", "add", "dev", "--api-key", "my_secret_token"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(out["status"], "added");
    assert_eq!(out["name"], "dev");
    assert_eq!(out["identity"], "test@tic.io");

    // 2. List accounts
    let (code, out, _) = env.json(&["accounts", "list"]);
    assert_eq!(code, 0);
    assert_eq!(out["count"], 1);
    assert_eq!(out["accounts"][0]["name"], "dev");
    assert_eq!(out["accounts"][0]["keyStatus"], "stored");

    // 3. Test account
    let (code, out, _) = env.json(&["accounts", "test", "dev"]);
    assert_eq!(code, 0);
    assert_eq!(out["name"], "dev");
    assert_eq!(out["keyStatus"], "valid");

    // 4. Remove account
    let (code, out, _) = env.json(&["accounts", "remove", "dev", "--yes"]);
    assert_eq!(code, 0);
    assert_eq!(out["status"], "removed");

    // 5. List accounts after remove
    let (code, out, _) = env.json(&["accounts", "list"]);
    assert_eq!(code, 0);
    assert_eq!(out["count"], 0);
}

#[test]
fn company_get() {
    let mock = Mock::start(vec![(
        "GET",
        "search/companies?q=5567926687&query_by=registrationNumber&per_page=1",
        200,
        json!({
            "found": 1,
            "hits": [
                {
                    "document": {
                        "companyId": 3325421,
                        "registrationNumber": "556792-6687",
                        "name": "Bosma Interactive AB"
                    }
                }
            ]
        }),
    )]);
    let env = Env::new(&mock).with_account();

    let (code, out, err) = env.json(&["company", "get", "556792-6687", "-a", "work"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(out["companyId"], 3325421);
    assert_eq!(out["registrationNumber"], "556792-6687");
}

#[test]
fn company_get_by_id() {
    let mock = Mock::start(vec![(
        "GET",
        "search/companies?q=*&query_by=registrationNumber&filter_by=companyId%3A%5B3325421%5D&per_page=1",
        200,
        json!({
            "found": 1,
            "hits": [
                {
                    "document": {
                        "companyId": 3325421,
                        "name": "Bosma Interactive AB"
                    }
                }
            ]
        }),
    )]);
    let env = Env::new(&mock).with_account();

    let (code, out, err) = env.json(&["company", "get-by-id", "3325421", "-a", "work"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(out["companyId"], 3325421);
    assert_eq!(out["name"], "Bosma Interactive AB");
}

#[test]
fn company_search() {
    let mock = Mock::start(vec![(
        "GET",
        "search/companies?q=Volvo&query_by=names.nameOrIdentifier&per_page=10",
        200,
        json!({
            "found": 2,
            "hits": [
                {"document": {"name": "Volvo AB", "registrationNumber": "556012-5790"}},
                {"document": {"name": "Volvo Car AB", "registrationNumber": "556810-8988"}}
            ]
        }),
    )]);
    let env = Env::new(&mock).with_account();

    let (code, out, err) = env.json(&["company", "search", "Volvo", "-a", "work"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(out["found"], 2);
    assert_eq!(out["items"].as_array().unwrap().len(), 2);
}

#[test]
fn company_intelligence_and_datasets() {
    let mock = Mock::start(vec![
        (
            "GET",
            "datasets/companies/3325421/intelligence",
            200,
            json!({"companyId": 3325421, "warnings": [], "riskScore": "low"}),
        ),
        (
            "GET",
            "datasets/companies/3325421/graph?includeBeneficialOwner=true&maxTotalNodes=10000",
            200,
            json!({"nodes": [{"id": 1, "label": "Owner"}], "edges": []}),
        ),
        (
            "GET",
            "datasets/companies/3325421/se/beneficial-owners",
            200,
            json!([{"name": "Niels Bosma", "percentage": 100}]),
        ),
    ]);
    let env = Env::new(&mock).with_account();

    let (code, out, err) = env.json(&["company", "intelligence", "3325421", "-a", "work"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(out["riskScore"], "low");

    let (code, out, err) = env.json(&["company", "graph", "3325421", "-a", "work"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(out["nodes"][0]["label"], "Owner");

    let (code, out, err) = env.json(&["company", "beneficial-owners", "3325421", "-a", "work"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(out[0]["name"], "Niels Bosma");
}

#[test]
fn person_search_and_get() {
    let mock = Mock::start(vec![
        (
            "GET",
            "datasets/persons?query=198207174171&pageSize=100",
            200,
            json!([{"personId": 1625054, "personalIdentityNumber": "198207174171"}]),
        ),
        ("GET", "datasets/persons/1625054", 200, json!({"personId": 1625054, "name": "Niels Bosma"})),
    ]);
    let env = Env::new(&mock).with_account();

    let (code, out, err) = env.json(&["person", "search", "198207174171", "-a", "work"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(out[0]["personId"], 1625054);

    let (code, out, err) = env.json(&["person", "get", "1625054", "-a", "work"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(out["name"], "Niels Bosma");
}

#[test]
fn vehicle_search() {
    let mock = Mock::start(vec![(
        "GET",
        "search/vehicles/se?q=ABC123&query_by=licencePlate&per_page=10",
        200,
        json!({
            "found": 1,
            "hits": [{"document": {"licencePlate": "ABC123", "make": "Volvo"}}]
        }),
    )]);
    let env = Env::new(&mock).with_account();

    let (code, out, err) = env.json(&["vehicle", "search", "ABC123", "-a", "work"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(out["found"], 1);
    assert_eq!(out["items"][0]["licencePlate"], "ABC123");
}

#[test]
fn bankruptcy_search() {
    let mock = Mock::start(vec![(
        "GET",
        "search/companies/bankruptcies/se?q=5566778899&query_by=registrationNumber&per_page=10",
        200,
        json!({
            "found": 1,
            "hits": [{"document": {"registrationNumber": "5566778899", "status": "active"}}]
        }),
    )]);
    let env = Env::new(&mock).with_account();

    let (code, out, err) = env.json(&["bankruptcy", "search", "5566778899", "-a", "work"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(out["found"], 1);
    assert_eq!(out["items"][0]["status"], "active");
}

#[test]
fn direct_api_key_flag_works_without_account() {
    let mock = Mock::start(vec![(
        "GET",
        "search/companies?q=5567926687&query_by=registrationNumber&per_page=1",
        200,
        json!({
            "found": 1,
            "hits": [{"document": {"companyId": 3325421}}]
        }),
    )]);
    let env = Env::new(&mock);

    let (code, out, err) = env.json(&["company", "get", "556792-6687", "--api-key", "direct_token_123"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(out["companyId"], 3325421);
}

#[test]
fn upstream_api_error_translation() {
    let mock = Mock::start(vec![
        ("GET", "datasets/persons/999999", 404, json!({"error": "Person not found"})),
        ("GET", "datasets/persons/888888", 401, json!({"error": "Invalid API key"})),
        ("GET", "datasets/persons/777777", 429, json!({"error": "Too many requests"})),
        ("GET", "datasets/persons/666666", 500, json!({"error": "Internal server error"})),
    ]);
    let env = Env::new(&mock).with_account();

    let (code, _, err) = env.json(&["person", "get", "999999", "-a", "work"]);
    assert_eq!(code, 4);
    assert_eq!(err["code"], "not_found");

    let (code, _, err) = env.json(&["person", "get", "888888", "-a", "work"]);
    assert_eq!(code, 3);
    assert_eq!(err["code"], "auth_required");

    let (code, _, err) = env.json(&["person", "get", "777777", "-a", "work"]);
    assert_eq!(code, 5);
    assert_eq!(err["code"], "rate_limited");

    let (code, _, err) = env.json(&["person", "get", "666666", "-a", "work"]);
    assert_eq!(code, 2);
    assert_eq!(err["code"], "network");
}
