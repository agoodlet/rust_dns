use configparser::ini::Ini;
use domain::base::{name::Dname, Rtype};
use domain::rdata::AllRecordData;
use domain::resolv::StubResolver;
use std::{collections::HashMap, env, str::FromStr};

// I don't think this needs to be a result anymore
fn load_config() -> Result<
    (
        HashMap<String, Option<String>>,
        HashMap<String, Option<String>>,
    ),
    std::io::Error,
> {
    let mut config = Ini::new();
    let map = config.load("config.ini").unwrap();

    let record_types = map["record_types"].clone();
    let subdomains = map["subdomains"].clone();

    Ok((record_types, subdomains))
}

fn resolve_record(domain: Dname<Vec<u8>>, record: Rtype) -> Vec<String> {
    let res = StubResolver::run(move |stub| async move { stub.query((domain, record)).await });
    // I have to do this over two lines otherwise I get some weird error with temporary values
    // or some shit
    let res = res.unwrap();
    let res = res.answer().unwrap().limit_to::<AllRecordData<_, _>>();
    let mut out = Vec::new();
    for r in res {
        let r = r.unwrap();
        let r = r.data().to_string();
        out.push(r);
    }
    out
}

fn main() {
    let config = load_config().expect("Could not load config file");

    let mut args = env::args().skip(1);
    let name = args
        .next()
        .and_then(|arg| Dname::<Vec<_>>::from_str(&arg).ok());
    let name = match name {
        Some(name) => name,
        _ => {
            println!("Usage: command <domain>");
            return;
        }
    };

    let mut records: Vec<_> = Vec::<Rtype>::new();

    let record_types = json::from(config.0.clone());
    for record in record_types.entries() {
        let record = Rtype::from_str(record.0).expect("unable to resolve Rtype from string");
        records.push(record);
    }

    records.sort_by(|a, b| a.to_string().cmp(&b.to_string()));

    println!("Domain: {}\n", name);

    let subdomains = json::from(config.1.clone());
    for subdomain in subdomains.entries() {
        // we just want to resolve the A record for these
        let resolved = resolve_record(name.clone(), Rtype::A);
        print!("{}{} > ", subdomain.0, name);
        for rec in resolved {
            println!("{}", rec);
        }
    }

    print!("\n");
    for record in records {
        let name = name.clone();
        let contents = resolve_record(name, record);
        // if the record is NS
        // resolve the a record for each content
        for content in contents {
            print!("{} {} ", record, content);
            // might be worth implementing a hostname validator
            // then if the content is a hostname, resolve the A record
            if record == Rtype::Ns {
                let ns_resolved = resolve_record(Dname::from_str(&content).unwrap(), Rtype::A);
                print!(" > {}\n", ns_resolved[0]);
            }
        }
        print!("\n");
    }
}
