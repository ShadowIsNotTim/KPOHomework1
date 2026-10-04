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

fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::new().run(terminal))
}

type RcPSt = Rc<dyn PageState>;

trait PageState {
    fn new() -> RcPSt where Self: Sized;
    fn draw(&self, frame: &mut Frame, sc: &mut ServiceCenter);
    fn handle_key_event(&mut self, key_event: KeyEvent, sc: &mut ServiceCenter);
    fn back(&self, sc: &mut ServiceCenter) -> RcPSt; // Esc pressed
    fn move_to(&self, sc: &mut ServiceCenter) -> Option<RcPSt>;
}


trait PageMenuState : PageState {
    fn items_list(&self) -> Vec<String>;
    fn items_cnt(&self) -> usize;
    fn cur_item(&self) -> usize;
    fn set_cur_item(&self, new_cur: usize);
    fn move_to_item(&self, sc: &mut ServiceCenter);
    fn title(&self) -> String;
    fn draw(&self, frame: &mut Frame, sc: &mut ServiceCenter) {
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
    fn handle_key_event(&mut self, key_event: KeyEvent, sc: &mut ServiceCenter) {
        match key_event.code {
            KeyCode::Up => self.set_cur_item(std::cmp::max(self.cur_item() - 1, 0)),
            KeyCode::Down => self.set_cur_item(std::cmp::min(self.items_cnt() - 1, self.cur_item() + 1)),
            KeyCode::Enter => self.move_to_item(sc),
            _ => {}
        }
    }
}


trait PageFormState<'a> : PageState {
    fn title(&self) -> String;
    fn active_textarea_mut(&mut self) -> &mut TextArea<'a>;
    fn textarea_list(&self) -> &[TextArea<'a>];
    fn handle_key_event(&mut self, key: KeyEvent, sc: &mut ServiceCenter) {
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

impl PageState for ExitPage {
    fn new() -> RcPSt {
        Rc::new(ExitPage)
    }
    fn draw(&self, frame: &mut Frame, sc: &mut ServiceCenter) {

    }
    fn handle_key_event(&mut self, key_event: KeyEvent, sc: &mut ServiceCenter) {

    }
    fn back(&self, sc: &mut ServiceCenter) -> RcPSt {
        PageMainMenu::new()
    }
    fn move_to(&self, sc: &mut ServiceCenter) -> Option<RcPSt> {
        Option::None
    }
}


struct PageMainMenu {
    move_to_ret: Option<Rc<PageState>>,
    options_fn: Vec<fn() -> RcPSt>,
    options: Vec<String>,
    cur: usize
}


impl PageState for PageMainMenu {
    fn new(&self) -> RcPSt {
        let a: Vec<RcPSt> = {
            ExitPage::new
        };
        let b: Vec<String> = {
            "выход (exit)"
        };
        PageMainMenu {
            move_to_ret: None,
            options_fn: a,
            options: b,
            cur: 0
        }
    }
}

impl PageMenuState for PageMainMenu {
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
    fn move_to_item(&self, sc: &mut ServiceCenter) {
        self.move_to_ret = self.options_fn[self.cur](sc);
    }
    fn title(&self) -> String {
        "Главное меню"
    }
    fn move_to(&self, sc: &mut ServiceCenter) -> Option<Rc<PageState>> {
        self.move_to_ret
    }
    fn back(&self, sc: &mut ServiceCenter) -> Self {
        PageMainMenu::new();
    }
}


#[derive(Debug, Default)]
struct App {
    sc: ServiceCenter,
    page: dyn PageState
}

impl App {
    fn new() -> Self {
        let sc = ServiceCenter::new();
        let page = PageMainMenu::new();
        Self { sc, page }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while self.page != ExitPage {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
            let r = self.page.move_to(&mut self.sc);
            self.page = r.unwrap_or_default(self.page);
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
