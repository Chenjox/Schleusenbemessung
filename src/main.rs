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

use crate::hydraulic::*;

struct FuellRechteck {
    oeffnungsgeschwindigkeit: f64, // in m/s
    breite: f64,
    hoehe: f64,
}

fn dhyd(area: f64, umfang: f64) -> f64 {
    return 4.0 * area / umfang;
}

impl Fuellquerschnitt for FuellRechteck {
    fn querschnitt(&self, zeit: f64) -> f64 {
        let temp = zeit * self.oeffnungsgeschwindigkeit;
        return if temp > self.hoehe {
            self.breite * self.hoehe
        } else {
            self.breite * temp
        };
    }

    fn freigegebene_hoehe(&self, zeit: f64) -> f64 {
        return (zeit * self.oeffnungsgeschwindigkeit).min(self.hoehe);
    }

    fn freigegebene_breite(&self, _hoehe: f64) -> f64 {
        return self.breite;
    }

    fn is_fully_opened(&self, zeit: f64) -> bool {
        return zeit * self.oeffnungsgeschwindigkeit > self.hoehe;
    }

    fn durchflussverslust_ueberfall(
        &self,
        _schleuse: &Schleuse,
        _pot_hoehe: f64,
        unterehoehe: f64,
        zeit: f64,
    ) -> f64 {
        //Äquivalente QS Breite
        let b = self.querschnitt(zeit) / (self.freigegebene_hoehe(zeit) - unterehoehe);
        let x = (self.freigegebene_hoehe(zeit) - unterehoehe) / b;
        return 0.673 + x * (-0.0511667 + x * (-0.0105 + x * (-0.047333 + x * (0.018))));
    }

    fn durchflussverslust_unterstroemung(
        &self,
        schleuse: &Schleuse,
        _pot_hoehe: f64,
        unterehoehe: f64,
        zeit: f64,
    ) -> f64 {
        //Einfluss
        let z1: f64 = 0.5;
        //Verengung
        let areafull = self.breite * self.hoehe;
        let areafree = self.breite * self.freigegebene_hoehe(zeit);
        let z2 = 0.5
            * (1.0
                - dhyd(areafull, 2.0 * (self.breite + self.hoehe))
                    / dhyd(
                        areafree,
                        2.0 * (self.breite * self.freigegebene_hoehe(zeit)),
                    ))
            .powi(2);
        // Ausweitung
        let kammerwasserspiegel =
            unterehoehe + schleuse.oberhaupt.oberwassersohle - schleuse.unterhaupt.unterwassersohle;
        let z3 = 1.2
            * (1.0
                - dhyd(
                    areafree,
                    2.0 * (self.breite * self.freigegebene_hoehe(zeit)),
                ) / dhyd(
                    schleuse.kammer.breite * kammerwasserspiegel,
                    (schleuse.kammer.breite) + 2.0 * kammerwasserspiegel,
                ))
            .powi(2);
        return (1.0) / ((1.0 + z1 + z2.max(0.0) + z3.max(0.0)).sqrt());
    }
} // I = 0.018 x^4 + -0.047333 x^3 - 0.0105 x^2 - 0.0511667 x^1 + 0.673

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
        fuellquerschnitt: Box::new(FuellRechteck {
            oeffnungsgeschwindigkeit: 0.003,
            breite: 4.0,
            hoehe: 1.250,
        }),
    };

    let fuell2 = Fuellquerschnittssystem {
        hoehe: 0.04,
        fuellquerschnitt: Box::new(FuellRechteck {
            oeffnungsgeschwindigkeit: 0.003,
            breite: 4.0,
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
            querschnitte: vec![Box::new(fuell1), Box::new(fuell2)],
        },
    };
    info!("Running Simulation");
    let v = schleuse.fuell_schleuse();
    let v = v
        .iter()
        .map(|i| i.map(|e| e.to_string()).join(","))
        .collect::<Vec<String>>()
        .join("\n");

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
