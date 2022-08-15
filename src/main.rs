mod bruteforce;
mod hydraulic;

use serde::Deserialize;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use log::LevelFilter;
use log::{error, info, warn};
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
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))
        .unwrap();
    log4rs::init_config(config).unwrap();

    return Ok(());
}

fn erschaffe_schleuse(
    schleuse: &Schleusenwerte,
    hoehe: f64,
    breite: f64,
    fuellzeit: f64,
) -> Schleuse {
    let fuell1 = Fuellquerschnittssystem {
        hoehe: 0.04,
        startzeit: 0.0,
        fuellquerschnitt: Box::new(FuellRechteck {
            oeffnungsgeschwindigkeit: fuellzeit,
            breite: breite,
            hoehe: hoehe,
        }),
    };

    let fuell2 = Fuellquerschnittssystem {
        hoehe: 0.04,
        startzeit: 0.0,
        fuellquerschnitt: Box::new(FuellRechteck {
            oeffnungsgeschwindigkeit: fuellzeit,
            breite: breite,
            hoehe: hoehe,
        }),
    };

    let fuell3 = Fuellquerschnittssystem {
        hoehe: 0.04,
        startzeit: 0.0,
        fuellquerschnitt: Box::new(FuellRechteck {
            oeffnungsgeschwindigkeit: fuellzeit,
            breite: breite,
            hoehe: hoehe,
        }),
    };

    let fuell4 = Fuellquerschnittssystem {
        hoehe: 0.04,
        startzeit: 0.0,
        fuellquerschnitt: Box::new(FuellRechteck {
            oeffnungsgeschwindigkeit: fuellzeit,
            breite: breite,
            hoehe: hoehe,
        }),
    };

    let schleusen = Schleuse {
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
    //info!("Running Simulation");
    //let result = schleuse.fuell_schleuse();
    //let result = result.last().unwrap().zeitschritt;

    return schleusen;
}

fn rechne_schleuse(schl: &Schleuse) -> f64 {
    let result = schl.fuell_schleuse();
    let result = result.last().unwrap().zeitschritt;
    return result;
}

fn auswertung_wasserspiegelneigung(schl: &Schleuse, res: &Vec<Simulationsschritt>) -> f64 {
    let erg = res
        .iter()
        .map(|s| {
            //let wellengeschwindigkeit = (s.kammerwasserspiegel * 9.81).sqrt();
            let wasserspiegelneigung =
                s.durchflusszunahme / (schl.kammer.breite * 4.0 * 9.81) * 1000.0;
            wasserspiegelneigung
        })
        .fold(0.0, |max, val: f64| val.max(max));
    return erg;
}

fn auswertung_fuelloeffnung(schl: &Schleuse, res: &Vec<Simulationsschritt>) -> usize {
    let mut count = 0;
    let erg = res.iter().filter(|&f| !f.events.is_empty()).for_each(|f| {
        let ev = &f.events;
        let oeffen = ev
            .iter()
            .filter(|eve| eve.status == FuellsystemStatus::VollGeoeffnet)
            .count();
        count += oeffen
    });

    return count;
}

fn ausprobieren(
    schleuse: Schleusenwerte,
    vgesch: (f64, f64),
    vhoehe: (f64, f64),
    vbreite: (f64, f64),
) {
    let var_geschwindigkeit = vgesch;
    let var_hoehe = vhoehe;
    let var_breite = vbreite;

    for v in (0..100).step_by(10) {
        let geschwi = var_geschwindigkeit.0
            + (var_geschwindigkeit.1 - var_geschwindigkeit.0) * v as f64 / 100.0;
        let mut results: Vec<[f64; 5]> = Vec::new();
        for i in (0..100).step_by(2) {
            let hoehe = var_hoehe.0 + (var_hoehe.1 - var_hoehe.0) * i as f64 / 100.0;
            for j in (0..100).step_by(2) {
                let breite = var_breite.0 + (var_breite.1 - var_breite.0) * j as f64 / 100.0;
                let schleus = erschaffe_schleuse(&schleuse, hoehe, breite, geschwi);

                let r = schleus.fuell_schleuse();
                let time = r.last().unwrap().zeitschritt;
                let wasserspiegel = auswertung_wasserspiegelneigung(&schleus, &r);
                let offnung = auswertung_fuelloeffnung(&schleus, &r);
                results.push([hoehe, breite, time, wasserspiegel, offnung as f64]);
            }
        }
        // Finden des minimums
        {
            let mut min = f64::INFINITY;
            let mut index_min = 0;
            for c in 0..results.len() {
                if min > results[c][0] {
                    if 60.0 * 21.0 > results[c][2] {
                        if 0.4 > results[c][3] {
                            min = results[c][0];
                            index_min = c;
                        }
                    }
                }
            }
            println!(
                "Minimale Höhe bei with v = {} m/s : h = {}, b = {}",
                geschwi, results[index_min][0], results[index_min][1]
            )
        }
        //
        let r = results
            .iter()
            .map(|f| format!("{},{},{},{},{}", f[0], f[1], f[2], f[3], f[4]))
            .collect::<Vec<String>>()
            .join("\n");
        let nam = format!("dimen{:03}.csv", v);
        let path = Path::new(&nam);
        let mut file = match File::create(&path) {
            Err(why) => {
                error!("Couldn't create dimen{:03}.csv: {}", nam, why);
                return;
            }
            Ok(file) => file,
        };
        match file.write_all(r.as_bytes()) {
            Err(why) => error!("couldn't write to dimen{}.csv: {}", nam, why),
            Ok(_) => info!("successfully wrote to dimen{}.csv", nam),
        }
    }
}

fn simuliere_schleuse(schl: &Schleuse) {
    info!("Durchrechnen der Schleuse");
    let v = schl.fuell_schleuse();
    let mut events = Vec::new();
    for k in &v {
        if !k.events.is_empty() {
            for event in &k.events {
                events.push(K(k.zeitschritt, String::from(&event.desc)));
                //println!("{:?},{:?}", k.zeitschritt, event);
            }
        }
    }
    info!("Auswerten der Ergebnisse");
    let max_k = auswertung_wasserspiegelneigung(schl, &v);
    println!("I_w = {} mm/m", max_k);
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
        Ok(_) => info!("successfully wrote to events.csv"),
    }
    let path = Path::new("result.csv");
    let mut file = match File::create(&path) {
        Err(why) => panic!("Couldn't create result.csv: {}", why),
        Ok(file) => file,
    };
    match file.write_all(v.as_bytes()) {
        Err(why) => panic!("couldn't write to result.csv: {}", why),
        Ok(_) => info!("successfully wrote to result.csv"),
    }
}

struct K(f64, String);

fn main() {
    match setup_logger() {
        Ok(_) => {}
        Err(_) => panic!("Logging doesn't work"),
    };
    info!("Set up logger");
    info!("Reading File 'test.toml'");
    let schleuse = match read_schleusenwerte("test.toml") {
        Ok(s) => s,
        Err(s) => panic!("{:?}", s),
    };
    // Variieren der einzelnen Werte

    let var_geschwindigkeit = (0.0005, 0.005);
    let var_hoehe = (0.3, 0.6);
    let var_breite = (1.0, 2.3);
    ausprobieren(schleuse, var_geschwindigkeit, var_hoehe, var_breite)

    //let v = erschaffe_schleuse(&schleuse, 1.2, 1.0, 0.001);
    //simuliere_schleuse(&v)
    //println!("{}", v)
}
