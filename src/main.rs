use std::{io::Write};
use std::fs;
use reqwest::{StatusCode};
use scraper::{Html, Selector};
use std::path::Path;
use regex::Regex;

mod utils;

#[tokio::main]
async fn main() {

    // initial link
    let mut url = String::from("https://practicalguidetoevil.wordpress.com/2015/03/25/prologue/");

    let mut document = new_chapter(&url).await;

    // integer needed to secure chronology is kept. Anyone reading this is free to
    // implement something that analyzes substrings to categorize and label files
    let mut page_n = 0;

    // loops from reading the document to querying for the next html document
    loop {
        url = scraper(&document, &url, &page_n);

        println!("current url: {}", &url);

        // breaks when there is no more links to follow
        if url == "No more link :(" { break;}

        document = new_chapter(&url).await;

        page_n += 1;
    }
    // an exception, peregrine-I does not occur when going from link to link.
    // url = String::from("https://practicalguidetoevil.wordpress.com/2018/12/03/peregrine-i/");
    // document = new_chapter(&url).await;
    // let exception = scraper(&document, &url, &page_n);
    // println!("{}", &exception);

}

fn scraper(document: &Html, url: &String, page_n: &i32) -> String {

// all pertinent tags that are used to select sections of html
    let raw_page = Selector::parse("article.post").unwrap();
    let navlink = Selector::parse("div.nav-next").unwrap();
    let link = Selector::parse("a").unwrap();
    let pagetext = Selector::parse("div.entry-content").unwrap();
    let span_select = Selector::parse("span").unwrap();
    let pp = Selector::parse("p").unwrap();
    let heading = Selector::parse("h1.entry-title").unwrap();
    let cite = Selector::parse("cite").unwrap();

// subsection made for future looping through
    let maintext = document.select(&pagetext).next().unwrap();

    let heading = format!("# {} \n\n", document.select(&heading).next().unwrap().inner_html());
    
    // Text is initialized with the fitting heading, 
    let mut textpiece = String::from(&heading);

    let mut citation = match document.select(&cite).next() {
        Some(citing) => citing.inner_html(),
        None => String::from(""),
    };

    for element in maintext.select(&pp) {
        // checks if <p> contains any internal <span>
        if element.select(&span_select).next().is_some() {

            // patchjob because beginning quotes falls out of <span>
            if element.inner_html().starts_with('“') {
                textpiece.push('“');
            }

            // loops through any span within 
            for element in element.select(&span_select) {
                textpiece.push_str(&element.inner_html());   
                textpiece.push_str("\n\n");
            }
        } else {
        // same as spans, less hassle tho
        textpiece.push_str(&element.inner_html());
        textpiece.push_str("\n\n");

        if citation != "" {
            textpiece.push_str(&citation);
            textpiece.push_str("\n\n");
            citation = String::from("")
        }

        }
    }

    // Cleanup Section: mostly because there is the nasty &nbsp; everywhere in the books
    let re = Regex::new(r"&nbsp;").unwrap();

    textpiece = re.replace_all(&textpiece, " ").to_string();
    // Document Section: the text string gets written to file, also makes a raw html file for posterity

    // Creating two paths for, one for a clean chapter and one for a raw html dump
    let title = format!("[{}]{}.md", &page_n, &url[54..url.len()-1]); 
    let raw_title = format!("raw_html[{}] {}.html", &page_n, &url[54..url.len()-1]);

    //^ While it would have been preferable to use the normal title, 
    // windows files don't support : which features in almost every chapter
    // patch solution is to simply use the url, leaving out the last / for compatibility

    let unrefined_doc = document.select(&raw_page).next().unwrap().html();

    let home = Path::new("Chapters");

    let mut raw_data = home.join("raw chapters");
    let mut clean_chapter = home.join("PGTE Chapters");

    fs::create_dir_all(&clean_chapter).unwrap();
    fs::create_dir_all(&raw_data).unwrap();

    clean_chapter.push(&title);
    raw_data.push(&raw_title);
    
    let mut chapter = fs::File::create(&clean_chapter).unwrap();
    let mut garble = fs::File::create(&raw_data).unwrap();
    
    write!(&mut garble, "{}", String::from(&unrefined_doc)).unwrap();
    write!(&mut chapter, "{}", String::from(&textpiece)).unwrap(); 


// Link Section: all the scraping and file writing is done, this moves on to the next chapter

// Specifies the section for links, the <a> link itself has no predicatble signifiers
// this below is needed in case there is no link, so the program doesn't end by panic
    let linkcat = match document.select(&navlink).next() {
        Some(linkref) => linkref,
        None => return String::from("No more link :("),
        
    };

// the url is printed out
    String::from(linkcat.select(&link).next().unwrap().value().attr("href").unwrap())

}

// searches for a webpage with the url given, return the full html doc
async fn new_chapter(url: &str) -> Html {

    let client = utils::get_client();

    let result = client.get(url).send().await.unwrap();

    let raw_html = match result.status() {
        StatusCode::OK => result.text().await.unwrap(),
        _ => panic!("Something went wrong"),
    };

    Html::parse_document(&raw_html)
}

/*
useful snippets for making your own little additions to this

    Taking a single instance of text:
    let single_text: String = <Html from website>.select(&<Insert Selector Type>).next().unwrap().inner_html());
    
    Taking multiple text blocks in a section:
        for element in maintext.select(&pp) {
        

        // checks if <p> contains any internal <span>
        if element.select(&span_select).next() != None {

            // patchjob because beginning quotes falls out of <span>
            if element.inner_html().starts_with("“") {
                textpiece.push_str("“");
            }

            // loops through any span within 
            for element in element.select(&span_select) {
                textpiece.push_str(&element.inner_html());   
                textpiece.push_str("\n\n");
            }
        } else {
            
        println!("p: {}\n", element.inner_html());
        }

        
    }
 */