
use warp::reject::Reject;
use std::fmt;

#[derive(Debug)]
pub struct Problem {
    status_code: u16,
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HTTP StatusCode: {}", self.status_code)
    }
}


impl Reject for Problem {}

pub fn from_anyhow(e: anyhow::Error, status_code: u16) -> Problem {
    let _ = match e.downcast::<Problem>() {
        Ok(problem) => return problem,
        Err(e) => e,
    };

    Problem {
        status_code
    }
}
