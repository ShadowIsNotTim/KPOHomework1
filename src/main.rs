use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use tui_textarea::TextArea;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Stylize, Style},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget, List, Borders},
    DefaultTerminal, Frame,
};
use std::rc::Rc;

use strum::{Display, EnumIter, IntoEnumIterator};
use strum::{VariantArray};

mod service_center;
use service_center::ServiceCenter;

use crate::PageOption::{AddNewTransport, MainMenu};

fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::new().run(terminal))
}

trait IPage {
    fn draw(&self, frame: &mut Frame, sc: &mut ServiceCenter);
    fn handle_key_event(&mut self, key_event: KeyEvent, sc: &mut ServiceCenter);
    fn back(&self) -> PageOption;
    fn move_to(&self, sc: &mut ServiceCenter) -> Action;
}

struct ListPage {
    options: Vec<(String, Action)>,
    cur: usize,
    title: String,
    backpage: PageOption,
    moving: Action
}

impl IPage for ListPage {
    fn draw(&self, frame: &mut Frame, sc: &mut ServiceCenter) {

    }
    fn handle_key_event(&mut self, key_event: KeyEvent, sc: &mut ServiceCenter) {
        Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    KeyCode::Up => cur -= 1,
                    KeyCode::Down => cur += 1,
                    KeyCode::Enter | KeyCode::Space | KeyCode::Right => {
                        self.moving = self.options[cur].1
                    }
                }
            }
    }
    fn back(&self) -> PageOption {
        self.backpage
    }
    fn move_to(&self, sc: &mut ServiceCenter) -> Action {
        self.moving
    }
}

struct FormPage {
    title: String
}

struct TextPage {
    text: String
}

enum Page {
    List(ListPage),
    Form(FormPage),
    Text(TextPage)
}

impl IPage for Page {
    fn draw(&self, frame: &mut Frame, sc: &mut ServiceCenter) {
        match self {
            Page::List(p) => p.draw(frame, sc),
            Page::Form(p) => p.draw(frame, sc),
            Page::Text(p) => p.draw(frame, sc)
        }
    }
    fn handle_key_event(&mut self, key_event: KeyEvent, sc: &mut ServiceCenter) {
        match self {
            Page::List(p) => p.handle_key_event(key_event, sc),
            Page::Form(p) => p.handle_key_event(key_event, sc),
            Page::Text(p) => p.handle_key_event(key_event, sc)
        }
    }
    fn back(&self) -> PageOption {
        match Self {
            Page::List(p) => p.back(),
            Page::Form(p) => p.back(),
            Page::Text(p) => p.back()
        }
    }
    fn move_to(&self, sc: &mut ServiceCenter) -> Action {
        match self {
            Page::List(p) => p.move_to(sc),
            Page::Form(p) => p.move_to(sc),
            Page::Text(p) => p.move_to(sc)
        }
    }
}

enum Action {
    None,
    Back,
    Quit,
    Go(PageOption)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, Display, Default)]
enum PageOption {
    #[strum(to_string = "главное меню")]
    #[default]
    MainMenu,
    #[strum(to_string = "добавить новый транспорт")]
    AddNewTransport,
    #[strum(to_string = "провести техосмотр")]
    TechReview,
    #[strum(to_string = "статистика потребления энергии")]
    EnergyStatistic,
    #[strum(to_string = "список устройств для новичков")]
    NubList,
    #[strum(to_string = "инвентарь")]
    Inventory,
    #[strum(to_string = "о нас")]
    AboutUs,
    #[strum(to_string = "выход (exit)")]
    Exit
}


// #[derive(Debug, Default)]
struct App {
    sc: ServiceCenter,
    page: Page
}

impl App {
    fn new() -> Self {
        let sc = ServiceCenter::new();
        let page = ListPage::new();
        Self { sc, page }
    }

    fn build_page(page: PageOption) -> Page {
        match page {
            MainMenu => Page::List({
                {},
                0,
                "главное меню",
                PageOption::MainMenu
            }),
            AddNewTransport => todo!()
        }
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
            let r = self.page.move_to(&mut self.sc);
            match r {
                Action::Back => self.page = build_page(self.page.back()),
                Action::None => {},
                Action ::Quit => return Ok(()),
                Action::Go(p) => self.page = build_page(p)
            }
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        self.page.draw(frame, &mut self.sc)
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    KeyCode::Esc => self.page = self.page.back(&mut self.sc),
                    _ => self.page.handle_key_event(key_event, &mut self.sc),
                }
            }
            _ => {}
        };
        Ok(())
    }
}
