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

use core::ServiceCenter;

fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::new().run(terminal))
}

trait PageState {
    fn new(sc: ServiceCenter) -> Self;
    fn linkname() -> String;
    fn draw(&self, frame: &mut Frame);
    fn handle_key_event(&mut self, key_event: KeyEvent);
    fn back() -> Rc<Self>; // Esc pressed
    fn move_to(&self) -> Option<Rc<Self>>;
}


trait PageMenuState : PageState {
    fn items_list(&self) -> Vec<String>;
    fn items_cnt(&self) -> usize;
    fn cur_item(&self) -> usize;
    fn set_cur_item(&self, new_cur: usize);
    fn move_to_item(&self);
    fn title(&self) -> String;
    fn draw(&self, frame: &mut Frame) {
        let items: Vec<String> = self.items_list()
        .iter()
        .map(|option| {
            option.to_string()
        })
        .collect();

        let list = List::new(items)
            .block(Block::default().title(self.title()).borders(Borders::ALL))
            .highlight_style(Style::default().bg(ratatui::style::Color::Yellow).fg(ratatui::style::Color::Black))
            .highlight_symbol(">> ");

        frame.render_widget(list, frame.area());
    }
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Up => self.set_cur_item(std::cmp::max(self.cur_item() - 1, 0)),
            KeyCode::Down => self.set_cur_item(std::cmp::min(self.items_cnt() - 1, self.cur_item() + 1)),
            KeyCode::Enter => self.move_to_item(),
            _ => {}
        }
    }
    fn back() -> Self;
    fn is_quited(&self) -> bool;
}


trait PageFormState<'a> : PageState {
    fn title(&self) -> String;
    fn active_textarea_mut(&mut self) -> &mut TextArea<'a>;
    fn textarea_list(&self) -> 
    fn draw(&self, frame: &mut Frame);
    fn next_focus(&self);
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Enter | KeyCode::Tab => {
                    self.next_focus();
                }
                _ => {
                    self.active_textarea_mut().input(key);
                }
            }
        }
    }
    fn back() -> Self;
    fn is_quited(&self) -> bool;
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, Display, Default)]
enum MainMenuPagesOption {
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
    Exit(ExitPage)
}

struct ExitPage;

impl ExitPage for PageState {
    fn new(sc: ServiceCenter) -> Self {
        {}
    }
    fn linkname() -> String {
        "выход (exit)"
    }
    fn draw(&self, frame: &mut Frame) {

    }
    fn handle_key_event(&mut self, key_event: KeyEvent) {

    }
    fn back() -> Rc<Self> {
        PageMainMenu::new()
    }
    fn move_to(&self) -> Option<Rc<Self>> {
        Option::None
    }
}


struct PageMainMenu {
    sc: ServiceCenter,
    move_to_ret: Option<Rc<PageState>>,
    enum_options: Vec<MainMenuPagesOption>,
    options: Vec<String>,
    cur: usize
}


impl PageMainMenu for PageMenuState {
    fn linkname() {
        "в главное меню"
    }
    fn new(sc: ServiceCenter) -> Self {
        {
            sc,
            Option::None,
            MainMenuPagesOption::iter().skip(1).collect(),
            MainMenuPagesOption::iter().skip(1).map(|el| el.to_string()).collect(),
            0
        }
    }
    fn items_list(&self) -> Vec<String> {
        self.options
    }
    fn items_cnt(&self) -> usize {
        self.options.len();
    }
    fn cur_item(&self) -> usize {
        self.cur
    }
    fn set_cur_item(&self, new_cur: usize) {
        self.cur = new_cur;
    }
    fn move_to_item(&self);
    fn title(&self) -> String {
        "Главное меню"
    }
    fn move_to(&self) -> Option<Rc<PageState>> {
        self.move_to_ret
    }
    fn back() -> Self {
        PageMainMenu::new(self.sc);
    }
}


#[derive(Debug, Default)]
struct App {
    page: PageState
}

impl App {
    fn new() -> Self {
        Self {
            page: PageMainMenu::new(ServiceCenter::new()),
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while self.page != ExitPage {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
            let r = self.page.move_to();
            self.page = r.unwrap_or_default(self.page);
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        self.page.draw(frame)
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    KeyCode::Esc => self.page = self.page.back(),
                    _ => self.page.handle_key_event(key_event),
                }
            }
            _ => {}
        };
        Ok(())
    }
}
