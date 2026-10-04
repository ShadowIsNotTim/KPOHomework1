use std::rc::Rc;

pub struct KWh(pub f32);
pub struct Wh(pub f32);
pub struct Mm(pub f32);
pub struct Code3Num(pub u8);


// VEHICLES

pub trait Vehicle {
    // интерфейс для транспортных средств
    fn get_id(&self) -> u32;
    fn get_energy(&self) -> KWh;
    fn get_capacity(&self) -> KWh;
    fn get_consumption(&self) -> KWh;

    // Киловатт-часы (KWh) (K большая тк codestyle)
    // увы в тз киловатты именно
    // по этому пришлось флоаты узать

    fn simplicity(&self) -> u8;
    fn for_beginners(&self) -> bool;
    fn get_model(&self) -> String;
    fn name(&self) -> String {
        return format!("{} id-{}", self.get_model(), self.get_id());
    }
}


struct ElectricScooter {
    id: u32,
    model: String,
    energy: KWh,
    capacity: KWh,
    wheel_diameter: Mm
}


struct ElectricBike {
    id: u32,
    model: String,
    energy: KWh,
    capacity: KWh,
    wheel_diameter: Mm,
    has_passenger_seat: bool
}


// ITEMS

struct ChargingStation {
    id: u32,
    location_code: String,
    port_count: u8,
    occupied_ports: u8,
    charge_speed: Wh
}

enum ClothingSize {
    S,
    M,
    L,
    XL
}

struct Helmet {
    id: u32,
    size: ClothingSize,
    is_damaged: bool
}

struct SmartLock {
    id: u32,
    battery_level_percent: u8,
    is_locked: bool,
    code: Code3Num
}




// Service Center

pub struct ServiceCenter {
    techs: Vec<Rc<dyn Vehicle>>
}

impl ServiceCenter {
    pub fn new() -> Self {
        Self { techs: Vec::new() }
    }
}