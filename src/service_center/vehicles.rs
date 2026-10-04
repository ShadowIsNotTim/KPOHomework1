use super::{AreaVec, ServiceCenter, KWh};

pub struct Mm(pub f32);


pub trait Vehicle {
    // интерфейс для транспортных средств
    fn get_id(&self) -> u32;
    fn name(&self) -> String;
    fn get_energy(&self) -> KWh;
    fn get_capacity(&self) -> KWh;
    fn get_consumption(&self) -> KWh;

    // Киловатт-часы (KWh) (K большая тк codestyle)
    // увы в тз киловатты именно
    // по этому пришлось флоаты узать

    fn simplicity(&self) -> u8;
    fn for_beginners(&self) -> bool;
    fn get_model(&self) -> String;
    fn to_string(&self) -> String {
        return format!("{} id-{}", self.get_model(), self.get_id());
    }
    fn build_areas(areas: &mut AreaVec) where Self: Sized;
    fn done(areas: &AreaVec, sc: &mut ServiceCenter) where Self: Sized;
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
