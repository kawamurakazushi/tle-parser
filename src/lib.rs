extern crate nom;

use nom::{
    bytes::complete::{tag, take, take_until},
    combinator::{map, map_opt, map_parser, map_res, opt, rest},
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

// 36258-4 => 0.36258e-4
fn ugly_float_parser(input: &str) -> IResult<&str, f64> {
    map_res(
        tuple((opt(tag("-")), take_until("-"), tag("-"), rest)),
        |(sign, a, _, b): (Option<&str>, &str, &str, &str)| {
            format!("{}0.{}e-{}", sign.unwrap_or(""), a, b).parse::<f64>()
        },
    )(input)
}

fn satellite_number_parser(input: &str) -> IResult<&str, u32> {
    map_res(take(5usize), |i: &str| i.parse::<u32>())(input)
}

fn one_space_parser(input: &str) -> IResult<&str, &str> {
    tag(" ")(input)
}

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
                _check_sum,
            ),
            _,
            // second line
            (
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
                revolution_number,
                _,
            ),
        ),
    ) = tuple((
        take_until("\n"),
        tag("\n"),
        // first line parser
        map_parser(
            take_until("\n"),
            tuple((
                tag("1"),
                one_space_parser,
                satellite_number_parser,
                map_opt(take(1usize), |i: &str| i.chars().nth(0usize)),
                one_space_parser,
                map(take(8usize), |i: &str| i.trim()),
                one_space_parser,
                map(take(14usize), |i: &str| i.trim()),
                one_space_parser,
                map_res(map(take(10usize), |i: &str| i.trim()), |i: &str| {
                    i.parse::<f64>()
                }),
                one_space_parser,
                map_parser(map(take(8usize), |i: &str| i.trim()), ugly_float_parser),
                one_space_parser,
                map_parser(map(take(8usize), |i: &str| i.trim()), ugly_float_parser),
                one_space_parser,
                map_res(take(1usize), |i: &str| i.parse::<u32>()),
                one_space_parser,
                map_res(map(take(4usize), |i: &str| i.trim()), |i: &str| {
                    i.parse::<u32>()
                }),
                map_res(take(1usize), |i: &str| i.parse::<u32>()),
            )),
        ),
        tag("\n"),
        // second line parser
        tuple((
            tag("2"),
            one_space_parser,
            satellite_number_parser,
            one_space_parser,
            map_res(map(take(8usize), |i: &str| i.trim()), |i: &str| {
                i.parse::<f64>()
            }),
            one_space_parser,
            map_res(map(take(8usize), |i: &str| i.trim()), |i: &str| {
                i.parse::<f64>()
            }),
            one_space_parser,
            map_res(take(7usize), |i: &str| format!("0.{}", i).parse::<f64>()),
            one_space_parser,
            map_res(map(take(8usize), |i: &str| i.trim()), |i: &str| {
                i.parse::<f64>()
            }),
            one_space_parser,
            map_res(map(take(8usize), |i: &str| i.trim()), |i: &str| {
                i.parse::<f64>()
            }),
            one_space_parser,
            map_res(map(take(11usize), |i: &str| i.trim()), |i: &str| {
                i.parse::<f64>()
            }),
            map_res(map(take(5usize), |i: &str| i.trim()), |i: &str| {
                i.parse::<u32>()
            }),
            map_res(take(1usize), |i: &str| i.parse::<u32>()),
        )),
    ))(raw_tle)
    .map_err(|e| {
        println!("🤔  Error - {}", e);
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
    fn parse_ugly_float() {
        let (_, f) = ugly_float_parser("36258-4").unwrap();
        assert_eq!(f, 0.36258e-4);

        let (_, f) = ugly_float_parser("00000-0").unwrap();
        assert_eq!(f, 0.0);

        let (_, f) = ugly_float_parser("-36258-4").unwrap();
        assert_eq!(f, -0.36258e-4);
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
        assert_eq!(tle.element_number, 999);
        // 2nd line
        assert_eq!(tle.inclination, 97.7009);
        assert_eq!(tle.right_ascension, 312.6237);
        assert_eq!(tle.eccentricity, 0.0003899);
        assert_eq!(tle.argument_of_perigee, 7.8254);
        assert_eq!(tle.mean_anomaly, 352.3026);
        assert_eq!(tle.mean_motion, 14.92889838);
        assert_eq!(tle.revolution_number, 6175);
    }

    #[test]
    fn parse_iss_tle() {
        let raw_tle = "ISS (ZARYA)
1 25544U 98067A   20045.18587073  .00000950  00000-0  25302-4 0  9990
2 25544  51.6443 242.0161 0004885 264.6060 207.3845 15.49165514212791";

        let expected = TLE {
            name: String::from("ISS (ZARYA)"),
            satellite_number: 25544,
            classification: 'U',
            international_designator: String::from("98067A"),
            epoch: String::from("20045.18587073"),
            first_derivative_mean_motion: 0.00000950,
            second_derivative_mean_motion: 0.0,
            drag_term: 0.25302e-4,
            ephemeris_type: 0,
            element_number: 999,
            inclination: 51.6443,
            right_ascension: 242.0161,
            eccentricity: 0.0004885,
            argument_of_perigee: 264.6060,
            mean_anomaly: 207.3845,
            mean_motion: 15.49165514,
            revolution_number: 21279,
        };

        let tle = parse(&raw_tle).unwrap();

        assert_eq!(tle, expected);
    }
}
