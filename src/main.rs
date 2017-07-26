#[macro_use]
extern crate nom;
extern crate rand;
extern crate linefeed;
extern crate term;
extern crate textwrap;

use nom::{line_ending, not_line_ending};

use std::io::prelude::*;
use std::fs::File;
use std::str;

use rand::Rng;

use linefeed::{Reader, ReadResult};

use textwrap::fill;

#[derive(Debug, PartialEq, Eq)]
struct Question<'s> {
    q: &'s str,
    a: bool,
}

named!(question<&[u8], Question>,
    do_parse!(
        answer: one_of!("wf") >>
        many1!(one_of!("\t ")) >>
        question: map_res!(not_line_ending, str::from_utf8) >>
        (Question{
            q: question,
            a: match answer {
                'w' => true,
                'f' => false,
                _ => panic!("unexpected answer"),
            }
        })
    )
);

named!(section<&[u8], (&str, Vec<Question>)>,
    do_parse!(
        title: map_res!(not_line_ending, str::from_utf8) >>
        line_ending >>
        questions: separated_list_complete!(line_ending, question) >>
        ((title, questions))
    )
);

named!(question_file<&[u8], Vec<(&str, Vec<Question>)> >,
       many0!(ws!(section))
);

#[cfg(test)]
mod tests {
    use super::*;
    use nom::IResult::*;

    #[test]
    fn test_question() {
        assert_eq!(
            question(b"w Sample Question."),
            Done(
                &b""[..],
                Question {
                    q: "Sample Question.",
                    a: true,
                }
            )
        );
    }

    #[test]
    fn test_section() {
        assert_eq!(
            section(b"SECTION ONE\nf The Life?\n\nSECTION EMPTY\nwno"),
            Done(
                &b"\n\nSECTION EMPTY\nwno"[..],
                ("SECTION ONE",
                 vec![Question {
                     q: "The Life?",
                     a: false,
                 }])
            )
        );
    }

    #[test]
    fn test_file() {
        assert_eq!(
            question_file(b"SECT A\nf How far?\nw How wide?\n\nSECT B\nw Is it cold outside?\n"),
            Done(
                &b""[..],
                vec![
                    ("SECT A", vec![
                        Question {
                            q: "How far?",
                            a: false,
                        },
                        Question {
                            q: "How wide?",
                            a: true,
                        }
                    ]),
                    ("SECT B", vec![
                        Question {
                            q: "Is it cold outside?",
                            a: true,
                        }
                    ])
                ]
            )
        );
    }
}

fn main() {
    // Read questions
    let content = {
        let mut file = File::open("../Multiple-Choice.txt").expect("cannot open question file");
        let mut content = String::new();
        file.read_to_string(&mut content).expect("cannot read question file");
        content
    };
    let mut questions = question_file(content.as_bytes()).to_full_result().expect("invalid syntax in question file");

    // Prepare and shuffle questions
    let mut questions: Vec<Question> = questions.drain(..).flat_map(|s| s.1).collect();
    let mut rng = rand::thread_rng();
    rng.shuffle(&mut questions);

    // Setup prompt
    let mut reader = Reader::new("question").unwrap();
    reader.set_prompt(">>> ");
    let mut t = term::stdout().unwrap();

    // Do it!
    while let Some(question) = questions.pop() {
        writeln!(t, "\n(remaining: {})", questions.len()).unwrap();
        t.attr(term::Attr::Bold).unwrap();
        writeln!(t, "\n{}", fill(question.q, 60)).unwrap();
        t.reset().unwrap();
        if let Ok(ReadResult::Input(input)) = reader.read_line() {
            let user = input.contains("w") || input.contains("y");
            if user != question.a {
                t.fg(term::color::RED).unwrap();
                println!("Wrong answer!");
                t.reset().unwrap();
            } else {
                t.fg(term::color::GREEN).unwrap();
                println!("Correct!");
                t.reset().unwrap();
            }
        } else {
            break;
        }
    }

}
