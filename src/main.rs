mod bruteforce;
mod hydraulic;

use serde::Deserialize;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use log::LevelFilter;
use log::{info, warn};
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;

use crate::bruteforce::*;
use crate::hydraulic::*;

#[derive(Deserialize)]
struct Schleusenwerte {
    unterwasser: f64,
    unterwassersohle: f64,
    oberwasser: f64,
    oberwassersohle: f64,
    kanalbreite: f64,
    kammerbreite: f64,
    kammerlaenge: f64,
}

fn read_schleusenwerte(file_name: &str) -> Result<Schleusenwerte, toml::de::Error> {
    // Ein wenig File IO
    let path = Path::new(file_name);
    let mut file = match File::open(&path) {
        Err(why) => panic!("Couldn't open {}: {}", file_name, why),
        Ok(file) => file,
    };
    // Beim Lesen kann auch viel schief gehen
    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => panic!("couldn't read {}: {}", file_name, why),
        Ok(_) => {}
    };
    // Und beim Parsen erst...
    let contents: Result<Schleusenwerte, _> = toml::from_str(&s);
    return contents;
}

fn setup_logger() -> Result<(), ()> {
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build("log/output.log")
        .unwrap();
    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(
            Root::builder()
                .appender("logfile")
                .build(LevelFilter::Trace),
        )
        .unwrap();
    log4rs::init_config(config).unwrap();

    return Ok(());
}

struct K(f64, String);

fn main() {
    match setup_logger() {
        Ok(_) => {}
        Err(_) => panic!("Logging doens't work"),
    };
    info!("Set up logger");
    info!("Reading File 'test.toml'");
    let schleuse = match read_schleusenwerte("test.toml") {
        Ok(s) => s,
        Err(s) => panic!("{:?}", s),
    };

    let fuell1 = Fuellquerschnittssystem {
        hoehe: 0.04,
        startzeit: 0.0,
        fuellquerschnitt: Box::new(FuellRechteck {
            oeffnungsgeschwindigkeit: 0.0015,
            breite: 2.0,
            hoehe: 1.250,
        }),
    };

    let fuell2 = Fuellquerschnittssystem {
        hoehe: 0.04,
        startzeit: 0.0,
        fuellquerschnitt: Box::new(FuellRechteck {
            oeffnungsgeschwindigkeit: 0.0015,
            breite: 2.0,
            hoehe: 1.250,
        }),
    };

    let fuell3 = Fuellquerschnittssystem {
        hoehe: 0.04,
        startzeit: 0.0,
        fuellquerschnitt: Box::new(FuellRechteck {
            oeffnungsgeschwindigkeit: 0.0015,
            breite: 2.0,
            hoehe: 1.250,
        }),
    };

    let fuell4 = Fuellquerschnittssystem {
        hoehe: 0.04,
        startzeit: 0.0,
        fuellquerschnitt: Box::new(FuellRechteck {
            oeffnungsgeschwindigkeit: 0.0015,
            breite: 2.0,
            hoehe: 1.250,
        }),
    };

    let schleuse = Schleuse {
        kammer: Schleusenkammer {
            breite: schleuse.kammerbreite,
            laenge: schleuse.kammerlaenge,
        },
        oberhaupt: Oberhaupt {
            oberwasser: schleuse.oberwasser,
            oberwasserbreite: schleuse.kanalbreite,
            oberwassersohle: schleuse.oberwassersohle,
        },
        unterhaupt: Unterhaupt {
            unterwasser: schleuse.unterwasser,
            unterwasserbreite: schleuse.kanalbreite,
            unterwassersohle: schleuse.unterwassersohle,
        },
        fuellsystem: Fuellsystem {
            querschnitte: vec![
                Box::new(fuell1),
                Box::new(fuell2),
                Box::new(fuell3),
                Box::new(fuell4),
            ],
        },
    };
    info!("Running Simulation");
    let v = schleuse.fuell_schleuse();
    let mut events = Vec::new();
    for k in &v {
        if !k.events.is_empty() {
            for event in &k.events {
                events.push(K(k.zeitschritt, String::from(&event.desc)));
                //println!("{:?},{:?}", k.zeitschritt, event);
            }
        }
    }
    let v = v
        .iter()
        .map(|i| {
            format!(
                "{},{},{},{},{}",
                i.iteration,
                i.zeitschritt,
                i.kammerwasserspiegel,
                i.durchfluss,
                i.durchflusszunahme
            )
        })
        .collect::<Vec<String>>()
        .join("\n");
    let events = events
        .iter()
        .map(|i| format!("{},{}", i.0, i.1))
        .collect::<Vec<String>>()
        .join("\n");

    let path = Path::new("events.csv");
    let mut file = match File::create(&path) {
        Err(why) => panic!("Couldn't create result.csv: {}", why),
        Ok(file) => file,
    };
    match file.write_all(events.as_bytes()) {
        Err(why) => panic!("couldn't write to result.csv: {}", why),
        Ok(_) => println!("successfully wrote to result.csv"),
    }
    let path = Path::new("result.csv");
    let mut file = match File::create(&path) {
        Err(why) => panic!("Couldn't create result.csv: {}", why),
        Ok(file) => file,
    };
    match file.write_all(v.as_bytes()) {
        Err(why) => panic!("couldn't write to result.csv: {}", why),
        Ok(_) => println!("successfully wrote to result.csv"),
    }
    //println!("{}", v)
}
