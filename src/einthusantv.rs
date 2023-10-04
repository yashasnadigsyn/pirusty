use cursive::utils::Counter;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, ACCEPT_LANGUAGE, USER_AGENT};
use std::collections::HashMap;
use scraper::{Html, Selector};
use std::fs;
use cursive::traits::With;
use cursive::view::{Resizable, Nameable, Scrollable};
use cursive::views::{Dialog, TextView, LinearLayout, DummyView, EditView, Button, SelectView, ListView, ProgressBar};
use cursive::Cursive;
use std::sync::Arc;
use std::rc::Rc;
use std::path::Path;
use std::thread::sleep;
use std::time;

fn get_movie_name(movie_name: &str, lang: &str, counter: Counter) -> Result<HashMap<String, Vec<String>> , reqwest::Error> {
    
    let mut port_no = String::new();
    if Path::new("tor_port.txt").try_exists().unwrap() {
        port_no = fs::read_to_string("tor_port.txt").unwrap();
    } else {
        port_no = String::from("9050");
    }
    let proxy = reqwest::Proxy::all(format!("socks5h://127.0.0.1:{}", port_no)).expect("error connecting to tor!");
    
    //fs::write("check0.txt", "working");
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT_LANGUAGE, "en-US,en;q=0.5".parse().unwrap());
    headers.insert(USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3".parse().unwrap());

    let client = Client::builder().proxy(proxy).default_headers(headers).build()?;
    
    let formatted_url = format!("https://einthusan.tv/movie/results/?lang={}&query={}", &lang.trim(), &movie_name);
    //fs::write("url.txt", &formatted_url);

    counter.tick(25);

    // Perform a POST request with the payload
    let response = client
        .get(&formatted_url)
        .send()?;
    counter.tick(10);
    let maybe_err = response.error_for_status_ref().err();
    match maybe_err {
        Some(e) => Err(e),
        None => {
            //fs::write("check1.txt", "check");
            let body = response.text()?;
            //fs::write("response.txt", &body);
            let document = Html::parse_document(&body);
            let selector = Selector::parse("section#UIMovieSummary").unwrap();
            let mut image_title_info: HashMap<String, Vec<String>> = HashMap::new();
            let mut image_vec = Vec::new();
            let mut title_vec = Vec::new();
            let mut info_vec = Vec::new();
            let mut url_vec = Vec::new();
            if let Some(section_element) = document.select(&selector).next() {
                let li_selector = Selector::parse("li").unwrap();
                let li_elements = section_element.select(&li_selector);
                for li_element in li_elements {
                    let image_selector = Selector::parse("img").unwrap();
                    let title_selector = Selector::parse("h3").unwrap();
                    let info_selector = Selector::parse("div.info p").unwrap();
                    let url_selector = Selector::parse("a.title").unwrap();
                    let image_element = li_element.select(&image_selector).next();
                    let title_element = li_element.select(&title_selector).next();
                    let info_element = li_element.select(&info_selector).next();
                    let url_element = li_element.select(&url_selector).next();
                    if let Some(image) = image_element {
                        image_vec.push(image.value().attr("src").unwrap().to_string());
                    }
                
                    if let Some(title) = title_element {
                        let title = title.text().collect::<String>();
                        //fs::write("title.txt", &title);
                        title_vec.push(title);
                    }
                
                    if let Some(info) = info_element {
                        let info = info.text().collect::<String>();
                        //fs::write("wde.txt", info.clone());
                        info_vec.push(info);
                    }
                    if let Some(url) = url_element {
                        url_vec.push(url.value().attr("href").unwrap().to_string());
                    }
                    counter.tick(20);
                }
            } else {
                println!("Not found");
            }
            //fs::write("check2.txt", "check");
            image_title_info.insert(String::from("image"), image_vec);
            image_title_info.insert(String::from("title"),title_vec);
            image_title_info.insert(String::from("info"), info_vec);
            image_title_info.insert(String::from("url"), url_vec);
            counter.tick(20);
            Ok(image_title_info)
        }
    }

}

pub fn choose_lang(siv: &mut Cursive) {

    let langs = "Hindi \nKannada \nTelugu \nTamil \nMalayalam \nBengali \nMarathi \nPunjabi";
    siv.pop_layer();
    let mut select = SelectView::new()
                                            .h_align(cursive::align::HAlign::Left)
                                            .autojump();
    select.add_all_str(langs.lines());
    select.set_on_submit(choose_movie);
    siv.add_layer(Dialog::around(select.scrollable())
                                .title("PiRusty")
                                .button("quit", |s| s.quit()));

}
pub fn choose_movie(siv: &mut Cursive, lang: &str) {

    siv.pop_layer();
    let lang_clone = Rc::new(lang.to_owned());
    siv.add_layer(Dialog::around(LinearLayout::vertical()
                                                        .child(TextView::new("Which movie do you want to watch?"))
                                                        .child(DummyView.fixed_height(1))
                                                        .child(EditView::new()
                                                                            .with_name("movie_name")
                                                                            .fixed_width(20)))
                                .title("PiRusty")
                                .button("Okay", move |s| {
                                    let name = s
                                        .call_on_name("movie_name", |view: &mut EditView| {
                                            view.get_content()
                                        }).unwrap();
                                        choose_from_list(s, &name, &lang_clone)
                                })
                                .button("quit", |s| s.quit()));
    
}

pub fn choose_from_list(siv: &mut Cursive, movie_name: &str, lang: &str) {
    if movie_name.is_empty() {
        siv.add_layer(Dialog::info("Please enter a movie name!"));
    } else {
    let clone_movie_name_for_continue = Rc::new(movie_name.to_owned());
    let clone_movie_name_for_direct = Rc::new(movie_name.to_owned());
    let lang_clone_tor = Rc::new(lang.to_owned());
    let lang_clone_direct = Rc::new(lang.to_owned());
    siv.pop_layer();
    siv.add_layer(Dialog::around(LinearLayout::vertical()
                                                        .child(TextView::new("Before proceeding, Do you want to use Tor or without Tor?"))
                                                        //.child(TextView::new("1. ScraperAPI (1000 requests free per month, very fast, https://www.scraperapi.com)"))
                                                        .child(TextView::new("1. Tor (free, need this for bypassing restrictions)"))
                                                        .child(TextView::new("2. Direct (free, use this if your country doesn't have much restrictions)"))
                                                        .child(TextView::new("Note: We use a default 9050 port. If you want to use other port, create a tor_port.txt file with just port number in it and save it in the same folder where the app is located."))
                                                        .child(DummyView.fixed_height(2))
                                                        .child(LinearLayout::horizontal()
                                                        .child(Button::new("Tor", move |siv| {
                                                            let clone_movie_name = clone_movie_name_for_continue.clone();
                                                            check_the_method(siv, clone_movie_name, "tor", "false", &lang_clone_tor);
                                                        }))
                                                        .child(DummyView.fixed_width(2))
                                                        .child(Button::new("Direct", move |siv | {
                                                            let clone_movie_name = clone_movie_name_for_direct.clone();
                                                            check_the_method(siv, clone_movie_name, "direct", "false", &lang_clone_direct)
                                                        }))
                                                        .child(DummyView.fixed_width(2))
                                                        .child(Button::new("quit", |s| s.quit()))))
                                .title("PiRusty"));
    }
}

//not using scraper api method
// fn check_api_key() -> String {
//     let path = Path::new("api_key.txt");
//     if path.exists() {
//         let api_key = fs::read_to_string("api_key.txt").unwrap();
//         api_key
//     } else {
//         "false".to_string()
//     }
// }

pub fn check_the_method(siv: &mut Cursive, movie_name: Rc<String>, via: &str, exists: &str, lang: &str) {
    if via == "tor" {
        siv.pop_layer();
        progress_till_we_get(siv, movie_name.as_str() , lang)

    } else {
        siv.pop_layer();
        siv.add_layer(TextView::new("Direct method is still in development."));
    }
}

// fn create_api_key(api_key: &str) {
//     let _ = //fs::write("api_key.txt", api_key);
// }

pub fn progress_till_we_get(siv: &mut Cursive, movie_name: &str, lang: &str) {

    let cb = siv.cb_sink().clone();
    let clone_movie_name = Arc::new(movie_name.to_owned());
    let clone_lang = Arc::new(lang.to_owned());
    siv.pop_layer();
    siv.add_layer(Dialog::around(ProgressBar::new().range(0, 100).with_task(
        move |counter| {
            counter.tick(25);
            let res = get_movie_name(&clone_movie_name, &clone_lang.clone(), counter);
            if let Err(err) = &res {
                let _ = fs::write("error.txt", err.to_string());
                //siv.add_layer(Dialog::info(""));
                cb.send(Box::new(show_error)).unwrap();
                return ;
            }
            cb.send(Box::new(move |siv| show_final_result_of_movies(siv, res.unwrap()))).unwrap();
        })
        .full_width()
    ).title("PiRusty"));
    siv.set_autorefresh(true);
}

fn show_error(siv: &mut Cursive) {

    let err = fs::read_to_string("error.txt").unwrap();
    siv.add_layer(Dialog::info(err));
    

}

pub fn show_final_result_of_movies(siv: &mut Cursive, res: HashMap<String, Vec<String>>) {

    siv.pop_layer();
    let title_vec = res.get("title").unwrap().clone();
    let url_vec = res.get("url").unwrap().clone();
    let info_vec = res.get("info").unwrap().clone();
    siv.add_layer(Dialog::new()
                                .title("PiRusty")
                                .button("quit", |s| s.quit())
                                .content(ListView::new()
                                                        .with(|listy| {
                                                            for i in 0..title_vec.len() {
                                                                listy.add_child(format!("{} -> {}", &title_vec[i], &info_vec[i]).as_str(), SelectView::new().popup().item_str(format!("{}", &url_vec[i])).item_str(format!("{}", &url_vec[i])).on_submit(get_link_of_movie));
                                                            }
                                                        })));
}                

pub fn get_link_of_movie(siv: &mut Cursive, movie_url: &str) {

    siv.pop_layer();
    let cb = siv.cb_sink().clone();
    let clone_movie_url = Arc::new(movie_url.to_owned());
    siv.add_layer(Dialog::around(ProgressBar::new().range(0, 100).with_task(
        move |counter| {
            //counter.tick(50);
            //let output = Command::new("yt-dlp").arg(format!("https://einthusan.tv{}",&clone_movie_url)).arg("--proxy").arg("socks5h://localhost:9050").arg("--get-url").output().expect("Error (maybe yt-dlp?)");
            //counter.tick(25);
            //let output = String::from_utf8_lossy(&output.stdout).to_string();
            
            let output = get_raw_url(clone_movie_url, counter);
            
            cb.send(Box::new(move |siv| open_website(siv, output.unwrap()))).unwrap();

        }
    ).full_width()));
    siv.set_autorefresh(true);
}

fn get_raw_url(movie_url: Arc<String>, counter: Counter) -> Result<String, reqwest::Error>{

    counter.tick(25);
    let client = Client::builder().build()?;
    counter.tick(10);

    let clone_movie_url = (*movie_url.clone()).clone();
    let main_url = "https://einthusandl.adaptable.app/indian?url=";
    let clone_movie_url = format!("{}https://einthusan.tv{}", main_url, clone_movie_url);
    //fs::write("url.txt", &clone_movie_url);
    
    // Perform a POST request with the payload
    let response = client
        .get(&clone_movie_url)
        .send()?;

    let maybe_err = response.error_for_status_ref().err();  
    
    match maybe_err {
        Some(e) => Err(e),
        None => {

            let body = response.text()?;
            Ok(body)
        }
    }  
}

fn open_website(siv: &mut Cursive, raw_url: String) {
    siv.pop_layer();
    siv.add_layer(Dialog::around(TextView::new("The movie url is in movie_url.txt and a web browser will open in 3s")).title("PiRusty").button("quit", |s| s.quit()));
    fs::write("movie_url.txt", &raw_url);
    sleep(time::Duration::from_secs(3));
    open::that(raw_url);


}
