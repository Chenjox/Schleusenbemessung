mod hydraulic;

use serde::Deserialize;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use crate::hydraulic::*;

struct FuellRechteck {
    oeffnungsgeschwindigkeit: f64, // in m/s
    breite: f64,
    hoehe: f64,
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
}

#[derive(Deserialize)]
struct Schleusenwerte {
    unterwasser: f64,
    unterwassersohle: f64,
    oberwasser: f64,
    oberwassersohle: f64,
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

fn main() {
    let schleuse = match read_schleusenwerte("test.toml") {
        Ok(s) => s,
        Err(s) => panic!("{:?}", s),
    };

    let fuell1 = Fuellquerschnittssystem {
        hoehe: 0.3,
        fuellquerschnitt: Box::new(FuellRechteck {
            oeffnungsgeschwindigkeit: 0.0003,
            breite: 4.0,
            hoehe: 3.0,
        }),
    };

    let fuell2 = Fuellquerschnittssystem {
        hoehe: 0.3,
        fuellquerschnitt: Box::new(FuellRechteck {
            oeffnungsgeschwindigkeit: 0.0003,
            breite: 4.0,
            hoehe: 3.0,
        }),
    };

    let schleuse = Schleuse {
        kammer: Schleusenkammer {
            breite: schleuse.kammerbreite,
            laenge: schleuse.kammerlaenge,
        },
        oberhaupt: Oberhaupt {
            oberwasser: schleuse.oberwasser,
            oberwassersohle: schleuse.oberwassersohle,
        },
        unterhaupt: Unterhaupt {
            unterwasser: schleuse.unterwasser,
            unterwassersohle: schleuse.unterwassersohle,
        },
        fuellsystem: Fuellsystem {
            querschnitte: vec![Box::new(fuell1), Box::new(fuell2)],
        },
    };

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
