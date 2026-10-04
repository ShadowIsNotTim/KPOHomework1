use std::rc::Rc;
use ratatui_textarea::TextArea;

use super::{AreaVec, ServiceCenter, KWh};
use super::super::{BuildAreasFnT, DoneFnT};
use strum::{Display, EnumIter};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mm(pub f32);


pub trait Vehicle {
    // интерфейс для транспортных средств
    fn get_id(&self) -> u32;
    fn name(&self) -> String;
    fn get_energy(&self) -> KWh;
    fn get_capacity(&self) -> KWh;

    // Киловатт-часы (KWh) (K большая тк codestyle)
    // увы в тз киловатты именно
    // по этому пришлось флоаты узать

    fn get_consumption(&self) -> KWh {
        KWh(0.0)
    }
    fn simplicity(&self) -> u8 {
        0
    }
    fn for_beginners(&self) -> bool {
        false
    }
    fn get_model(&self) -> String;
    fn to_string(&self) -> String {
        return format!("{} id-{}", self.get_model(), self.get_id());
    }
    fn build_areas(areas: &mut AreaVec) where Self: Sized;
    fn done(areas: &AreaVec, sc: &mut ServiceCenter) where Self: Sized;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ElectricScooter {
    id: u32,
    model: String,
    energy: KWh,
    capacity: KWh,
    wheel_diameter: Mm
}

#[derive(Debug, Clone, PartialEq)]
pub struct ElectricBike {
    id: u32,
    model: String,
    energy: KWh,
    capacity: KWh,
    wheel_diameter: Mm,
    has_passenger_seat: bool
}

#[derive(Debug, Clone, Copy, PartialEq, Display, EnumIter)]
pub enum VehicleTypes {
    #[strum(to_string = "электрический скутер")]
    ElectricScooter,
    #[strum(to_string = "электрический велик")]
    ElectricBike
}

impl VehicleTypes {
    pub fn unpack(self) -> (String, BuildAreasFnT, DoneFnT) {
        match self {
            VehicleTypes::ElectricScooter => (
                "Новый скутер".to_string(), ElectricScooter::build_areas, ElectricScooter::done
            ),
            VehicleTypes::ElectricBike => (
                "Новый байк".to_string(), ElectricBike::build_areas, ElectricBike::done
            )
        }
    }
}

fn parse_wh(areas: &AreaVec, ind: usize) -> f32 {
    areas[ind].1.lines().join("").trim().parse().unwrap_or(0.0)
}

impl Vehicle for ElectricScooter {
    fn get_id(&self) -> u32 {
        self.id
    }

    fn name(&self) -> String {
        format!("{} id-{}", self.model, self.id)
    }

    fn get_energy(&self) -> KWh {
        self.energy
    }

    fn get_capacity(&self) -> KWh {
        self.capacity
    }

    fn get_model(&self) -> String {
        self.model.clone()
    }

    fn build_areas(areas: &mut AreaVec) {
        areas.push(("Модель".to_string(), TextArea::default()));
        areas.push(("Заряд (kWh)".to_string(), TextArea::default()));
        areas.push(("Ёмкость (kWh)".to_string(), TextArea::default()));
        areas.push(("Диаметр колеса (мм)".to_string(), TextArea::default()));
    }

    fn done(areas: &AreaVec, sc: &mut ServiceCenter) {
        let id = sc.next_transport_id();
        sc.add_transport(Rc::new(ElectricScooter {
            id,
            model: areas[0].1.lines().join(""),
            energy: KWh(parse_wh(areas, 1)),
            capacity: KWh(parse_wh(areas, 2)),
            wheel_diameter: Mm(parse_wh(areas, 3)),
        }));
    }
}

impl Vehicle for ElectricBike {
    fn get_id(&self) -> u32 {
        self.id
    }

    fn name(&self) -> String {
        format!("{} id-{}", self.model, self.id)
    }

    fn get_energy(&self) -> KWh {
        self.energy
    }

    fn get_capacity(&self) -> KWh {
        self.capacity
    }

    fn get_model(&self) -> String {
        self.model.clone()
    }

    fn build_areas(areas: &mut AreaVec) {
        areas.push(("Модель".to_string(), TextArea::default()));
        areas.push(("Заряд (kWh)".to_string(), TextArea::default()));
        areas.push(("Ёмкость (kWh)".to_string(), TextArea::default()));
        areas.push(("Диаметр колеса (мм)".to_string(), TextArea::default()));
        areas.push(("Пассажирское место? (y/n)".to_string(), TextArea::default()));
    }

    fn done(areas: &AreaVec, sc: &mut ServiceCenter) {
        let id = sc.next_transport_id();
        let seat = areas[4].1.lines().join("");
        sc.add_transport(Rc::new(ElectricBike {
            id,
            model: areas[0].1.lines().join(""),
            energy: KWh(parse_wh(areas, 1)),
            capacity: KWh(parse_wh(areas, 2)),
            wheel_diameter: Mm(parse_wh(areas, 3)),
            has_passenger_seat: matches!(
                seat.trim().to_lowercase().as_str(),
                "y" | "yes" | "1" | "true" | "да"
            ),
        }));
    }
}
