use std::rc::Rc;
use ratatui_textarea::TextArea;
use serde::{Serialize, Deserialize};

use super::{AreaVec, ServiceCenter, KWh};
use super::super::{BuildAreasFnT, DoneFnT};
use strum::{Display, EnumIter};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mm(pub f32);

#[typetag::serde]
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
        self.simplicity() >= 6
    }
    fn get_model(&self) -> String;
    fn to_string(&self) -> String {
        return format!("{} id-{}", self.get_model(), self.get_id());
    }
    fn build_areas(areas: &mut AreaVec) where Self: Sized;
    fn done(areas: &AreaVec, sc: &mut ServiceCenter) where Self: Sized;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ElectricScooter {
    id: u32,
    model: String,
    energy: KWh,
    capacity: KWh,
    wheel_diameter: Mm
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ElectricBike {
    id: u32,
    model: String,
    energy: KWh,
    capacity: KWh,
    wheel_diameter: Mm,
    has_passenger_seat: bool
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bicycle {
    id: u32,
    model: String,
    wheel_diameter: Mm
}

#[derive(Debug, Clone, Copy, PartialEq, Display, EnumIter)]
pub enum VehicleTypes {
    #[strum(to_string = "электрический скутер")]
    ElectricScooter,
    #[strum(to_string = "электрический велик")]
    ElectricBike,
    #[strum(to_string = "простой велосипед")]
    Bicycle
}

impl VehicleTypes {
    pub fn unpack(self) -> (String, BuildAreasFnT, DoneFnT) {
        match self {
            VehicleTypes::ElectricScooter => (
                "Новый скутер".to_string(), ElectricScooter::build_areas, ElectricScooter::done
            ),
            VehicleTypes::ElectricBike => (
                "Новый байк".to_string(), ElectricBike::build_areas, ElectricBike::done
            ),
            VehicleTypes::Bicycle => (
                "Новый велосипед".to_string(), Bicycle::build_areas, Bicycle::done
            )
        }
    }
}

fn parse_wh(areas: &AreaVec, ind: usize) -> f32 {
    areas[ind].1.lines().join("").trim().parse().unwrap_or(0.0)
}

#[typetag::serde]
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

    fn to_string(&self) -> String {
        format!(
            "Самокат {} #{}: {}/{} kWh, колесо {} мм, простота {}",
            self.model, self.id, self.energy.0, self.capacity.0,
            self.wheel_diameter.0, self.simplicity()
        )
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

#[typetag::serde]
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

    fn to_string(&self) -> String {
        format!(
            "Байк {} #{}: {}/{} kWh, колесо {} мм, пассажир: {}, простота {}",
            self.model, self.id, self.energy.0, self.capacity.0, self.wheel_diameter.0,
            if self.has_passenger_seat { "да" } else { "нет" }, self.simplicity()
        )
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

#[typetag::serde]
impl Vehicle for Bicycle {
    fn get_id(&self) -> u32 {
        self.id
    }

    fn name(&self) -> String {
        format!("{} id-{}", self.model, self.id)
    }

    fn get_energy(&self) -> KWh {
        KWh(0.0)
    }

    fn get_capacity(&self) -> KWh {
        KWh(0.0)
    }

    fn simplicity(&self) -> u8 {
        7
    }

    fn get_model(&self) -> String {
        self.model.clone()
    }

    fn to_string(&self) -> String {
        format!(
            "Велосипед {} #{}: колесо {} мм, простота {}",
            self.model, self.id, self.wheel_diameter.0, self.simplicity()
        )
    }

    fn build_areas(areas: &mut AreaVec) {
        areas.push(("Модель".to_string(), TextArea::default()));
        areas.push(("Диаметр колеса (мм)".to_string(), TextArea::default()));
    }

    fn done(areas: &AreaVec, sc: &mut ServiceCenter) {
        let id = sc.next_transport_id();
        sc.add_transport(Rc::new(Bicycle {
            id,
            model: areas[0].1.lines().join(""),
            wheel_diameter: Mm(parse_wh(areas, 1)),
        }));
    }
}
