use cursive::theme::{BorderStyle, Palette};
use cursive::traits::With;
use cursive::view::Scrollable;
use cursive::views::{Dialog, TextView, SelectView};
use cursive::Cursive;

mod einthusantv;

fn main() {
    let mut siv = cursive::default();

    // Start with a nicer theme than default
    siv.set_theme(cursive::theme::Theme {
        shadow: true,
        borders: BorderStyle::Simple,
        palette: Palette::retro().with(|palette| {
            use cursive::theme::BaseColor::*;

            {
                // First, override some colors from the base palette.
                use cursive::theme::Color::TerminalDefault;
                use cursive::theme::PaletteColor::*;

                palette[Background] = TerminalDefault;
                palette[View] = TerminalDefault;
                palette[Primary] = White.dark();
                palette[TitlePrimary] = Blue.light();
                palette[Secondary] = Blue.light();
                palette[Highlight] = Blue.dark();
            }

            {
                // Then override some styles.
                use cursive::theme::Effect::*;
                use cursive::theme::PaletteStyle::*;
                use cursive::theme::Style;
                palette[Highlight] = Style::from(Blue.light()).combine(Bold);
            }
        }),
    });

    siv.add_layer(Dialog::around(TextView::new("Welcome to PiRusty! Watch any movie or series you like (still under development)"))
                                .title("PiRusty")
                                .button("Continue", choose_site)
                                .button("quit", |s| s.quit())
                );

    siv.run();
}

fn choose_site(siv: &mut Cursive) {
    let sites = include_str!("assets/sites.txt");
    let mut select = SelectView::new()
    .h_align(cursive::align::HAlign::Left)
    .autojump();
    select.add_all_str(sites.lines());
    select.set_on_submit(move |siv, title: &str| {

        if title.contains("einthusan.tv") {
            einthusantv::choose_lang(siv);
        } else if title.contains("fmoviesz.to") {
            siv.add_layer(Dialog::info("Still in development!"));
        }

    });
    siv.pop_layer();
    siv.add_layer(Dialog::around(select.scrollable())
                                .title("PiRusty")
                                .button("quit", |s| s.quit())
                                );
}
