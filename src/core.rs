
pub struct KWh(pub f32);
pub struct Wh(pub f32);
pub struct Mm(pub f32);
pub struct Code3Num(pub u8);


// VEHICLES

pub trait Vehicle {
    // интерфейс для транспортных средств
    pub fn get_id() -> u32 {
        return id;
    }

    pub fn get_energy() -> KWh {
        return energy;
    }
    pub fn get_capacity() -> KWh {
        return capacity;
    }
    pub fn get_consumption() -> KWh;

    // Киловатт-часы (KWh) (K большая тк codestyle)
    // увы в тз киловатты именно
    // по этому пришлось флоаты узать

    pub fn simplicity() -> u8;
    pub fn for_beginners() -> bool {
        return simplicity >= 6;
    }
    pub fn get_model() -> &str {
        return model;
    }
    pub fn name() -> &str {
        return format!("{} id-{}", get_model(), get_id());
    }
}


struct ElectricScooter {
    id: u32,
    model: &str,
    energy: KWh,
    capacity: Kwh,
    wheel_diameter: Mm
}


struct ElectricBike {
    id: u32,
    model: &str,
    energy: KWh,
    capacity: Kwh,
    wheel_diameter: Mm,
    has_passenger_seat: bool
}


// ITEMS

struct ChargingStation {
    id: u32,
    location_code: &str,
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
    techs: vec<Vehicle>
}