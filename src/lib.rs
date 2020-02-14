extern crate nom;
use nom::character::complete::digit1;
use nom::{
    bytes::complete::{tag, take, take_until},
    character::complete::space1,
    combinator::{map_opt, map_parser, map_res, rest},
    error::ErrorKind,
    sequence::tuple,
    IResult,
};
use std::error;
use std::fmt;

#[derive(Debug)]
pub struct TLEError;

pub type Result<T> = std::result::Result<T, TLEError>;

impl fmt::Display for TLEError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid TLE Format")
    }
}

impl error::Error for TLEError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

#[derive(Debug, PartialEq)]
pub struct TLE {
    pub name: String,
    pub satellite_number: u32,
    pub classification: char,
    pub international_designator: String,
    // TODO: DateTime<Utc>
    pub epoch: String,
    pub first_derivative_mean_motion: f64,
    pub second_derivative_mean_motion: f64,
    pub drag_term: f64,
    pub ephemeris_type: u32,
    pub element_number: u32,
    pub inclination: f64,
    pub right_ascension: f64,
    pub eccentricity: f64,
    pub argument_of_perigee: f64,
    pub mean_anomaly: f64,
    pub mean_motion: f64,
    pub revolution_number: u32,
}

fn float_parser(input: &str) -> IResult<&str, f64> {
    map_parser(
        take_until(" "),
        map_res(
            tuple((take_until("-"), tag("-"), rest)),
            |(a, _, b): (&str, &str, &str)| format!("0.{}e-{}", a, b).parse::<f64>(),
        ),
    )(input)
}

fn satellite_number_parser(input: &str) -> IResult<&str, u32> {
    map_res(take(5usize), |i: &str| i.parse::<u32>())(input)
}

// fn first_line_parser(input: &str) -> IResult<&str, u32> {
//     map_res(take(5usize), |i: &str| i.parse::<u32>())(input)
// }

pub fn parse(raw_tle: &str) -> Result<TLE> {
    let (
        _,
        (
            name,
            _,
            (
                _,
                _,
                satellite_number,
                classification,
                _,
                international_designator,
                _,
                epoch,
                _,
                first_derivative_mean_motion,
                _,
                second_derivative_mean_motion,
                _,
                drag_term,
                _,
                ephemeris_type,
                _,
                element_number,
            ),
            _,
            //
            _,
            _,
            _satellite_number,
            _,
            inclination,
            _,
            right_ascension,
            _,
            eccentricity,
            _,
            argument_of_perigee,
            _,
            mean_anomaly,
            _,
            mean_motion,
            _,
            revolution_number,
        ),
    ) = tuple((
        take_until::<&str, &str, (_, ErrorKind)>("\n"),
        tag("\n"),
        // first line parser
        map_parser(
            take_until("\n"),
            tuple((
                tag("1"),
                space1,
                satellite_number_parser,
                map_opt(take(1usize), |i: &str| i.chars().nth(0usize)),
                space1,
                take_until(" "),
                space1,
                take_until(" "),
                space1,
                map_res(take_until(" "), |i: &str| i.parse::<f64>()),
                space1,
                float_parser,
                space1,
                float_parser,
                space1,
                map_res(take(1usize), |i: &str| i.parse::<u32>()),
                space1,
                map_res(rest, |i: &str| i.parse::<u32>()),
            )),
        ),
        tag("\n"),
        // second line parser
        tag("2"),
        space1,
        satellite_number_parser,
        space1,
        map_res(take_until(" "), |i: &str| i.parse::<f64>()),
        space1,
        map_res(take_until(" "), |i: &str| i.parse::<f64>()),
        space1,
        map_res(take(7usize), |i: &str| format!("0.{}", i).parse::<f64>()),
        space1,
        map_res(take_until(" "), |i: &str| i.parse::<f64>()),
        space1,
        map_res(take_until(" "), |i: &str| i.parse::<f64>()),
        space1,
        map_res(take_until(" "), |i: &str| i.parse::<f64>()),
        space1,
        map_res(digit1, |i: &str| i.parse::<u32>()),
    ))(raw_tle)
    .map_err(|e| {
        println!("Error - {}", e);
        TLEError
    })?;

    Ok(TLE {
        name: String::from(name),
        satellite_number,
        classification,
        international_designator: String::from(international_designator),
        epoch: String::from(epoch),
        first_derivative_mean_motion,
        second_derivative_mean_motion,
        drag_term,
        ephemeris_type,
        element_number,
        inclination,
        right_ascension,
        eccentricity,
        argument_of_perigee,
        mean_anomaly,
        mean_motion,
        revolution_number,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_float() {
        let (_, f) = float_parser("36258-4 ").unwrap();
        assert_eq!(f, 0.36258e-4);

        let (_, f) = float_parser("00000-0 ").unwrap();
        assert_eq!(f, 0.0);
    }

    #[test]
    fn parse_grus_tle() {
        let raw_tle = "GRUS-1A
1 43890U 18111Q   20044.88470557  .00000320  00000-0  36258-4 0  9993
2 43890  97.7009 312.6237 0003899   7.8254 352.3026 14.92889838 61757";

        let tle = parse(raw_tle).unwrap();

        assert_eq!(tle.name, "GRUS-1A");
        assert_eq!(tle.satellite_number, 43890);
        assert_eq!(tle.classification, 'U');
        assert_eq!(tle.international_designator, "18111Q");
        assert_eq!(tle.epoch, "20044.88470557");
        assert_eq!(tle.first_derivative_mean_motion, 0.00000320);
        assert_eq!(tle.second_derivative_mean_motion, 0.0);
        assert_eq!(tle.drag_term, 0.36258e-4);
        assert_eq!(tle.ephemeris_type, 0);
        assert_eq!(tle.element_number, 9993);
        // 2nd line
        assert_eq!(tle.inclination, 97.7009);
        assert_eq!(tle.right_ascension, 312.6237);
        assert_eq!(tle.eccentricity, 0.0003899);
        assert_eq!(tle.argument_of_perigee, 7.8254);
        assert_eq!(tle.mean_anomaly, 352.3026);
        assert_eq!(tle.mean_motion, 14.92889838);
        assert_eq!(tle.revolution_number, 61757);
    }

    #[test]
    #[ignore]
    fn partial_equal() {
        let raw_tle = "GRUS-1A
1 43890U 18111Q   20044.88470557  .00000320  00000-0  36258-4 0  9993
2 43890  97.7009 312.6237 0003899   7.8254 352.3026 14.92889838 61757";

        let expected = TLE {
            name: String::from("GRUS-1A"),
            satellite_number: 1,
            classification: 'a',
            international_designator: String::from("test"),
            epoch: String::from("test"),
            first_derivative_mean_motion: 1.9,
            second_derivative_mean_motion: 1.9,
            drag_term: 1.9,
            ephemeris_type: 1,
            element_number: 1,
            inclination: 1.9,
            right_ascension: 1.9,
            eccentricity: 1.9,
            argument_of_perigee: 1.9,
            mean_anomaly: 1.9,
            mean_motion: 1.9,
            revolution_number: 1,
        };

        let tle = parse(&raw_tle).unwrap();

        assert_eq!(tle, expected);
    }
}
