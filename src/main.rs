use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders, List, ListState, Paragraph},
    DefaultTerminal, Frame,
};

use strum::Display;

use tui_textarea::TextArea;

mod service_center;
use service_center::ServiceCenter;

fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::new().run(terminal))
}

trait IPage {
    fn draw(&self, frame: &mut Frame, sc: &mut ServiceCenter);
    fn handle_key_event(&mut self, key_event: KeyEvent, sc: &mut ServiceCenter);
    fn back(&self) -> PageOption;
    fn move_to(&mut self, sc: &mut ServiceCenter) -> Action;
}

struct ListPage {
    options: Vec<(String, Action)>,
    cur: usize,
    title: String,
    backpage: PageOption,
    moving: Action,
}

impl ListPage {
    fn new(title: impl Into<String>, backpage: PageOption, options: Vec<(String, Action)>) -> Self {
        Self {
            options,
            cur: 0,
            title: title.into(),
            backpage,
            moving: Action::None,
        }
    }
}

impl IPage for ListPage {
    fn draw(&self, frame: &mut Frame, _sc: &mut ServiceCenter) {
        let items: Vec<String> = self.options.iter().map(|(label, _)| label.clone()).collect();

        let list = List::new(items)
            .block(Block::default().title(self.title.as_str()).borders(Borders::ALL))
            .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black))
            .highlight_symbol(">> ");

        let mut state = ListState::default();
        state.select(Some(self.cur));
        frame.render_stateful_widget(list, frame.area(), &mut state);
    }

    fn handle_key_event(&mut self, key_event: KeyEvent, _sc: &mut ServiceCenter) {
        if key_event.kind != KeyEventKind::Press {
            return;
        }

        let last = self.options.len().saturating_sub(1);
        match key_event.code {
            KeyCode::Up => self.cur = self.cur.saturating_sub(1),
            KeyCode::Down => self.cur = (self.cur + 1).min(last),
            KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Right => {
                if let Some((_, action)) = self.options.get(self.cur) {
                    self.moving = *action;
                }
            }
            _ => {}
        }
    }

    fn back(&self) -> PageOption {
        self.backpage
    }

    fn move_to(&mut self, _sc: &mut ServiceCenter) -> Action {
        std::mem::replace(&mut self.moving, Action::None)
    }
}

struct FormPage {
    title: String,
    backpage: PageOption
    // areas: Vec<TextArea>,
    // build_areas_fn: fn (&FormPage) -> ()
}

impl IPage for FormPage {
    fn draw(&self, _frame: &mut Frame, _sc: &mut ServiceCenter) {}
    fn handle_key_event(&mut self, _key_event: KeyEvent, _sc: &mut ServiceCenter) {}
    fn back(&self) -> PageOption {
        self.backpage
    }
    fn move_to(&mut self, _sc: &mut ServiceCenter) -> Action {
        Action::None
    }
}

struct TextPage {
    text: String,
    backpage: PageOption,
}

impl IPage for TextPage {
    fn draw(&self, frame: &mut Frame, _sc: &mut ServiceCenter) {
        let paragraph = Paragraph::new(self.text.clone())
            .block(Block::default().title("Текст").borders(Borders::ALL));
        frame.render_widget(paragraph, frame.area());
    }
    fn handle_key_event(&mut self, _key_event: KeyEvent, _sc: &mut ServiceCenter) {}
    fn back(&self) -> PageOption {
        self.backpage
    }
    fn move_to(&mut self, _sc: &mut ServiceCenter) -> Action {
        Action::None
    }
}

enum Page {
    List(ListPage),
    Form(FormPage),
    Text(TextPage),
}

impl IPage for Page {
    fn draw(&self, frame: &mut Frame, sc: &mut ServiceCenter) {
        match self {
            Page::List(p) => p.draw(frame, sc),
            Page::Form(p) => p.draw(frame, sc),
            Page::Text(p) => p.draw(frame, sc),
        }
    }
    fn handle_key_event(&mut self, key_event: KeyEvent, sc: &mut ServiceCenter) {
        match self {
            Page::List(p) => p.handle_key_event(key_event, sc),
            Page::Form(p) => p.handle_key_event(key_event, sc),
            Page::Text(p) => p.handle_key_event(key_event, sc),
        }
    }
    fn back(&self) -> PageOption {
        match self {
            Page::List(p) => p.back(),
            Page::Form(p) => p.back(),
            Page::Text(p) => p.back(),
        }
    }
    fn move_to(&mut self, sc: &mut ServiceCenter) -> Action {
        match self {
            Page::List(p) => p.move_to(sc),
            Page::Form(p) => p.move_to(sc),
            Page::Text(p) => p.move_to(sc),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Action {
    None,
    Back,
    Quit,
    Go(PageOption),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, Default)]
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
}

struct App {
    sc: ServiceCenter,
    page: Page,
}

impl App {
    fn new() -> Self {
        let sc = ServiceCenter::new();
        let page = Self::build_page(PageOption::MainMenu);
        Self { sc, page }
    }

    fn menu_item(page: PageOption) -> (String, Action) {
        (page.to_string(), Action::Go(page))
    }

    fn main_menu_options() -> Vec<(String, Action)> {
        vec![
            Self::menu_item(PageOption::AddNewTransport),
            Self::menu_item(PageOption::TechReview),
            Self::menu_item(PageOption::EnergyStatistic),
            Self::menu_item(PageOption::NubList),
            Self::menu_item(PageOption::Inventory),
            Self::menu_item(PageOption::AboutUs),
            ("выход (exit)".to_string(), Action::Quit),
        ]
    }

    fn build_page(page: PageOption) -> Page {
        match page {
            PageOption::MainMenu => Page::List(ListPage::new(
                "Главное меню",
                PageOption::MainMenu,
                Self::main_menu_options(),
            )),
            PageOption::AboutUs => Page::Text(TextPage {
                text: "О нас".to_string(),
                backpage: PageOption::MainMenu,
            }),
            PageOption::AddNewTransport => todo!(),
            PageOption::TechReview => todo!(),
            PageOption::EnergyStatistic => todo!(),
            PageOption::NubList => todo!(),
            PageOption::Inventory => todo!(),
        }
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
            match self.page.move_to(&mut self.sc) {
                Action::Back => self.page = Self::build_page(self.page.back()),
                Action::None => {}
                Action::Quit => return Ok(()),
                Action::Go(p) => self.page = Self::build_page(p),
            }
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        self.page.draw(frame, &mut self.sc)
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if let Event::Key(key_event) = event::read()? {
            if key_event.kind == KeyEventKind::Press {
                if key_event.code == KeyCode::Esc {
                    let back = self.page.back();
                    self.page = Self::build_page(back);
                } else {
                    self.page.handle_key_event(key_event, &mut self.sc);
                }
            }
        }
        Ok(())
    }
}
