use log::{debug, error, info, trace, warn};

const G: f64 = 9.81;

pub trait Fuellquerschnitt {
    // Fläche des geöffneten Querschnitts zu einem Zeitpunkt s
    fn querschnitt(&self, zeit: f64) -> f64;

    // Freigegebene Hohe des Querschnitts
    fn freigegebene_hoehe(&self, zeit: f64) -> f64;

    // Freigegebene Breite bei einer gewissen hoehe des Querschnitts
    fn freigegebene_breite(&self, hoehe: f64) -> f64;

    // Ob der Fülllquerschnitt vollständig geöffnet ist..
    fn is_fully_opened(&self, zeit: f64) -> bool;

    /**
    Berechnet den Durchflussverlust in abhängigkeit von den jeweiligen Bedingungen.
    */
    fn durchflussverslust_ueberfall(
        &self,
        schleuse: &Schleuse,
        pot_hoehe: f64,
        unterehoehe: f64,
        zeit: f64,
    ) -> f64;
    fn durchflussverslust_unterstroemung(
        &self,
        schleuse: &Schleuse,
        pot_hoehe: f64,
        unterehoehe: f64,
        zeit: f64,
    ) -> f64;

    // Quadratur zur Ermittlung des Durchflusses
    // Die Potentialhoehe ist anzugeben auf die untere Kante des Füllquerschnitts
    // Die Überstromte Höhe des Füllquerschnitts ist die obere Kante des Füllquerschnitts
    fn quadratur_durchfluss_unterstroemung(
        &self,
        pot_hoehe: f64,
        uberstroemte_hoehe: f64,
        zeit: f64,
    ) -> f64 {
        let frei_hoehe = self.freigegebene_hoehe(zeit);
        // Welche obere Grenze ist maßgebend?
        let ober = frei_hoehe.min(uberstroemte_hoehe);
        let n: u32 = 100;
        let schritt = (ober) / (n as f64);
        // Trapezformel!
        let unteregrenze =
            self.freigegebene_breite(0.0) * (2.0 * G * (pot_hoehe - uberstroemte_hoehe)).sqrt();
        let oberegrenze =
            self.freigegebene_breite(ober) * (2.0 * G * (pot_hoehe - uberstroemte_hoehe)).sqrt();
        let mut rest = 0.0;
        for i in 1..(n - 1) {
            rest += self.freigegebene_breite(schritt * i as f64)
                * (2.0 * G * (pot_hoehe - (uberstroemte_hoehe))).sqrt();
        }
        return (rest + unteregrenze + oberegrenze) * schritt;
    }

    fn quadratur_durchfluss_ueberfall(
        &self,
        pot_hoehe: f64,
        uberstroemte_hoehe: f64,
        zeit: f64,
    ) -> f64 {
        let frei_hoehe = self.freigegebene_hoehe(zeit);
        if frei_hoehe < uberstroemte_hoehe {
            // Ist der Querschnitt komplett uberfüllt, ist der Uberfall 0
            return 0.0;
        }
        let n = 100;
        let schritt = (frei_hoehe - uberstroemte_hoehe) / (n as f64);

        let unteregrenze = self.freigegebene_breite(uberstroemte_hoehe)
            * (2.0 * G * (pot_hoehe - uberstroemte_hoehe)).sqrt();
        let oberegrenze =
            self.freigegebene_breite(frei_hoehe) * (2.0 * G * (pot_hoehe - frei_hoehe)).sqrt();
        let mut rest = 0.0;
        for i in 1..(n - 1) {
            rest += self.freigegebene_breite(uberstroemte_hoehe + schritt * i as f64)
                * (2.0 * G * (pot_hoehe - (uberstroemte_hoehe + schritt * i as f64))).sqrt();
        }
        return (rest + unteregrenze + oberegrenze) * schritt;
    }
}

pub struct Schleusenkammer {
    pub breite: f64,
    pub laenge: f64,
}

pub struct Oberhaupt {
    pub oberwasser: f64,
    pub oberwasserbreite: f64,
    pub oberwassersohle: f64,
}

pub struct Fuellquerschnittssystem {
    pub hoehe: f64,     // Unterkante des Querschnitts ab Bezugshöhe
    pub startzeit: f64, // In Sekunden
    pub fuellquerschnitt: Box<dyn Fuellquerschnitt>,
}

impl Fuellquerschnittssystem {
    /**
    Berechnet den Durchfluss des Füllsystems abhängig von der Zeit und der Bezugshöhe der Kammer
    Ist Höhe hoch genug ist kombinierter Zufluss anzusetzen
    Die obere hoehe gibt die Höhe des OWs bzw. des Kammerwassers an. (= Potential von dem Ausgegangen wird.)
    Die untere hoehe gibt die Höhe des Kammerwassers bzw des UWs an. (= Ob Querschnitt teilweise rückgestaut ist.)
    Beide Höhen sind ausgehend von der Bezugshöhe angegeben.
    Weitere Konstruktive Maße sind der Schleuse zu entnehmen.
    */
    pub fn durchfluss(
        &self,
        schleuse: &Schleuse,
        oberehoehe: f64,
        unterehoehe: f64,
        zeit: f64,
    ) -> f64 {
        if zeit <= self.startzeit {
            return 0.0;
        }
        let pot_hoehe = oberehoehe - self.hoehe;
        let ueberstroemhoehe = (unterehoehe - self.hoehe).max(0.0);
        // Block für die Verluste
        let mu_a = self.fuellquerschnitt.durchflussverslust_ueberfall(
            schleuse,
            pot_hoehe,
            unterehoehe,
            zeit - self.startzeit,
        );
        let mu_s = self.fuellquerschnitt.durchflussverslust_unterstroemung(
            schleuse,
            pot_hoehe,
            unterehoehe,
            zeit - self.startzeit,
        );

        if unterehoehe < self.hoehe {
            trace!("mu_a, mu_s, mu_as: {:?},{:?},{:?}", mu_a, mu_s, 0.0);
            mu_a * self.fuellquerschnitt.quadratur_durchfluss_ueberfall(
                pot_hoehe,
                0.0,
                zeit - self.startzeit,
            )
        } else {
            let fuellhoehe = self
                .fuellquerschnitt
                .freigegebene_hoehe(zeit - self.startzeit)
                .min(ueberstroemhoehe);
            let mu_as = (mu_a
                * (self.fuellquerschnitt.freigegebene_hoehe(zeit) - fuellhoehe).max(0.0)
                + (mu_s * fuellhoehe).max(0.0))
                / self.fuellquerschnitt.freigegebene_hoehe(zeit);
            trace!("mu_a, mu_s, mu_as: {:?},{:?},{:?}", mu_a, mu_s, mu_as);
            mu_as
                * (self.fuellquerschnitt.quadratur_durchfluss_unterstroemung(
                    pot_hoehe,
                    ueberstroemhoehe,
                    zeit - self.startzeit,
                ) + self.fuellquerschnitt.quadratur_durchfluss_ueberfall(
                    pot_hoehe,
                    ueberstroemhoehe,
                    zeit - self.startzeit,
                ))
        }
    }

    fn ist_ueberstroemt(&self, unterehoehe: f64, zeit: f64) -> bool {
        return self.hoehe < unterehoehe;
    }

    fn ist_vollstandig_ueberstroemt(&self, unterehoehe: f64, zeit: f64) -> bool {
        return self.hoehe
            + self
                .fuellquerschnitt
                .freigegebene_hoehe(zeit - self.startzeit)
            < unterehoehe;
    }

    fn ist_geoffnet(&self, zeit: f64) -> bool {
        return zeit > self.startzeit;
    }

    fn ist_vollstandig_geoffnet(&self, zeit: f64) -> bool {
        return self.fuellquerschnitt.is_fully_opened(zeit - self.startzeit);
    }
}

pub struct Fuellsystem {
    pub querschnitte: Vec<Box<Fuellquerschnittssystem>>,
}

impl Fuellsystem {
    fn durchfluss(&self, schleuse: &Schleuse, unterehoehe: f64, oberehoehe: f64, zeit: f64) -> f64 {
        let mut res = 0.0;
        for i in &self.querschnitte {
            res += i.durchfluss(schleuse, oberehoehe, unterehoehe, zeit);
        }
        return res;
    }

    pub fn anzahl_fuellsysteme(&self) -> usize {
        self.querschnitte.len()
    }
    pub fn ist_ueberstroemt(&self, unterehoehe: f64, zeit: f64) -> Vec<FuellsystemStatus> {
        let mut vec = Vec::new();
        for i in &self.querschnitte {
            if i.ist_vollstandig_ueberstroemt(unterehoehe, zeit) {
                vec.push(FuellsystemStatus::VollUeberfuellt)
            } else if i.ist_ueberstroemt(unterehoehe, zeit) {
                vec.push(FuellsystemStatus::StartUeberfuellung)
            } else {
                vec.push(FuellsystemStatus::Unbekannt)
            }
        }
        return vec;
    }
    pub fn oeffnungsstatus(&self, zeit: f64) -> Vec<FuellsystemStatus> {
        let mut vec = Vec::new();
        for i in &self.querschnitte {
            if i.ist_vollstandig_geoffnet(zeit) {
                vec.push(FuellsystemStatus::VollGeoeffnet)
            } else if i.ist_geoffnet(zeit) {
                vec.push(FuellsystemStatus::StartOeffnung)
            } else {
                vec.push(FuellsystemStatus::Unbekannt)
            }
        }
        return vec;
    }
}
pub struct Unterhaupt {
    pub unterwasser: f64,
    pub unterwasserbreite: f64,
    pub unterwassersohle: f64,
}
pub struct Schleuse {
    pub kammer: Schleusenkammer,
    pub oberhaupt: Oberhaupt,
    pub unterhaupt: Unterhaupt,
    pub fuellsystem: Fuellsystem,
}

impl Oberhaupt {
    pub fn wasserspiegel(&self) -> f64 {
        self.oberwasser - self.oberwassersohle
    }
}

impl Unterhaupt {
    pub fn wasserspiegel(&self) -> f64 {
        self.unterwasser - self.unterwassersohle
    }
}

impl Schleusenkammer {
    pub fn grundflaeche(&self) -> f64 {
        self.breite * self.laenge
    }
}

#[derive(Debug)]
pub struct Event {
    pub desc: String,
    pub status: FuellsystemStatus,
}

pub struct Simulationsschritt {
    pub iteration: u32,
    pub zeitschritt: f64,
    pub kammerwasserspiegel: f64,
    pub durchfluss: f64,
    pub durchflusszunahme: f64,
    pub events: Vec<Event>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum FuellsystemStatus {
    Unbekannt,
    StartOeffnung,
    VollGeoeffnet,
    StartUeberfuellung,
    VollUeberfuellt,
}

impl Schleuse {
    fn hubhoehe(&self) -> f64 {
        self.oberhaupt.oberwasser - self.unterhaupt.unterwasser
    }

    fn wasservolumen(&self) -> f64 {
        self.kammer.grundflaeche() * (self.hubhoehe() + self.unterhaupt.wasserspiegel())
    }

    pub fn fuell_schleuse(&self) -> Vec<Simulationsschritt> {
        let mut kammerspiegel = self.unterhaupt.wasserspiegel();
        let zeitschritt = 1.0;
        let mut i = 1;
        let mut volume = self.kammer.grundflaeche() * kammerspiegel;
        let max_iterations = 20000;

        let mut result_vec = Vec::new();
        let mut durchfluss = 0.0;
        debug!(
            "The start values for iteration in fuell_schleuse are: HKA = {:?}, volume = {:?}",
            kammerspiegel, volume
        );

        let anzahl_fuellsys = self.fuellsystem.querschnitte.len();
        let mut statusueberfuellt_fuellsys = self.fuellsystem.ist_ueberstroemt(
            (kammerspiegel - (self.oberhaupt.oberwassersohle - self.unterhaupt.unterwassersohle))
                .max(0.0),
            0.0,
        );
        let mut statusoffen_fuellsys = self.fuellsystem.oeffnungsstatus(0.0);

        while kammerspiegel < self.oberhaupt.oberwasser - self.unterhaupt.unterwassersohle
            && i < max_iterations
        {
            kammerspiegel = volume / self.kammer.grundflaeche();
            let unterehoehe = (kammerspiegel
                - (self.oberhaupt.oberwassersohle - self.unterhaupt.unterwassersohle))
                .max(0.0);
            let oberehoehe = self.oberhaupt.wasserspiegel();
            let durchfluss_alt = durchfluss;
            durchfluss = self.fuellsystem.durchfluss(
                &self,
                unterehoehe,
                oberehoehe,
                zeitschritt * (i as f64),
            );
            let durchfluss = if durchfluss.is_nan() { 0.0 } else { durchfluss };
            volume += durchfluss * zeitschritt;

            //Sind irgendwelche Events eingetreten?
            let mut events = Vec::new();

            {
                // Droppen ist wichtig
                let momentanstroem = self
                    .fuellsystem
                    .ist_ueberstroemt(unterehoehe, zeitschritt * (i as f64));

                let momentanoeff = self.fuellsystem.oeffnungsstatus(zeitschritt * (i as f64));

                for i in 0..anzahl_fuellsys {
                    if statusoffen_fuellsys[i] != momentanoeff[i] {
                        match &momentanoeff[i] {
                            FuellsystemStatus::StartOeffnung => {
                                events.push(Event {
                                    desc: String::from("SG"),
                                    status: FuellsystemStatus::StartOeffnung,
                                });
                                statusoffen_fuellsys[i] = FuellsystemStatus::StartOeffnung
                            }
                            FuellsystemStatus::VollGeoeffnet => {
                                events.push(Event {
                                    desc: String::from("VG"),
                                    status: FuellsystemStatus::VollGeoeffnet,
                                });
                                statusoffen_fuellsys[i] = FuellsystemStatus::VollGeoeffnet
                            }
                            _ => {}
                        };
                    }
                    if statusueberfuellt_fuellsys[i] != momentanstroem[i] {
                        match &momentanstroem[i] {
                            FuellsystemStatus::StartUeberfuellung => {
                                events.push(Event {
                                    desc: String::from("SU"),
                                    status: FuellsystemStatus::StartUeberfuellung,
                                });
                                statusueberfuellt_fuellsys[i] =
                                    FuellsystemStatus::StartUeberfuellung;
                            }
                            FuellsystemStatus::VollUeberfuellt => {
                                events.push(Event {
                                    desc: String::from("VU"),
                                    status: FuellsystemStatus::VollUeberfuellt,
                                });
                                statusueberfuellt_fuellsys[i] = FuellsystemStatus::VollUeberfuellt;
                            }
                            _ => {}
                        }
                    }
                }
            }

            //let wellengeschwindigkeit = (kammerspiegel * G).sqrt();
            //let wasserspiegelneigung = (durchfluss - durchfluss_alt)
            //    / (zeitschritt
            //        * self.kammer.breite
            //        * wellengeschwindigkeit
            //        * wellengeschwindigkeit)
            //    * 10.0e3;

            result_vec.push(Simulationsschritt {
                iteration: i,
                zeitschritt: zeitschritt * f64::from(i),
                kammerwasserspiegel: kammerspiegel,
                durchfluss: durchfluss,
                durchflusszunahme: (durchfluss - durchfluss_alt) / zeitschritt,
                events: events,
            });

            i += 1;
        }

        return result_vec;
    }
}
