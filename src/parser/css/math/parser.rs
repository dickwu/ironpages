use cssparser::{Parser, Token};

use super::MAX_CALC_COMPLEXITY;
use super::ast::{
    AbsoluteLengthUnit, Angle, AngleUnit, BinaryExpression, BinaryOperation, ClampExpression, Flex,
    FontLengthUnit, Frequency, FrequencyUnit, Length, LengthUnit, LogExpression, MathExpression,
    MathFunction, MathLiteral, PairExpression, Resolution, ResolutionUnit, RoundExpression,
    RoundingStrategy, Time, TimeUnit, ViewportLengthUnit,
};

type ParseResult<T> = Result<T, cssparser::ParseError<()>>;

#[derive(Default)]
struct ParseLimits {
    terms: usize,
    nesting: usize,
}

pub(super) fn parse(source: &str) -> Option<MathExpression> {
    let mut parser = Parser::new(source);
    let mut limits = ParseLimits::default();
    parser
        .parse_entirely(|input| parse_sum(input, &mut limits))
        .ok()
}

fn parse_sum<'i>(input: &mut Parser<'i>, limits: &mut ParseLimits) -> ParseResult<MathExpression> {
    let mut expression = parse_product(input, limits)?;
    loop {
        let state = input.state();
        if input.expect_whitespace().is_err() {
            input.reset(&state);
            break;
        }
        let operation = match input.next() {
            Ok(Token::Delim('+')) => BinaryOperation::Add,
            Ok(Token::Delim('-')) => BinaryOperation::Subtract,
            _ => {
                input.reset(&state);
                break;
            }
        };
        input
            .expect_whitespace()
            .map_err(cssparser::ParseError::<()>::from)?;
        expression = binary(operation, expression, parse_product(input, limits)?);
    }
    Ok(expression)
}

fn parse_product<'i>(
    input: &mut Parser<'i>,
    limits: &mut ParseLimits,
) -> ParseResult<MathExpression> {
    let mut expression = parse_primary(input, limits)?;
    loop {
        let state = input.state();
        let operation = match input.next() {
            Ok(Token::Delim('*')) => BinaryOperation::Multiply,
            Ok(Token::Delim('/')) => BinaryOperation::Divide,
            _ => {
                input.reset(&state);
                break;
            }
        };
        expression = binary(operation, expression, parse_primary(input, limits)?);
    }
    Ok(expression)
}

fn parse_primary<'i>(
    input: &mut Parser<'i>,
    limits: &mut ParseLimits,
) -> ParseResult<MathExpression> {
    limits.terms += 1;
    if limits.terms > MAX_CALC_COMPLEXITY {
        return Err(cssparser::ParseError::custom(()));
    }
    let token = input.next()?.clone();
    let literal = match token {
        Token::Number { value, .. } => MathLiteral::Number(value),
        Token::Percentage { unit_value, .. } => MathLiteral::Percentage(unit_value * 100.0),
        Token::Dimension { value, unit, .. } => {
            if let Some(unit) = parse_length_unit(&unit) {
                MathLiteral::Length(Length { value, unit })
            } else if let Some(unit) = parse_angle_unit(&unit) {
                MathLiteral::Angle(Angle { value, unit })
            } else if let Some(unit) = parse_time_unit(&unit) {
                MathLiteral::Time(Time { value, unit })
            } else if let Some(unit) = parse_frequency_unit(&unit) {
                MathLiteral::Frequency(Frequency { value, unit })
            } else if let Some(unit) = parse_resolution_unit(&unit) {
                MathLiteral::Resolution(Resolution { value, unit })
            } else if unit.eq_ignore_ascii_case("fr") {
                MathLiteral::Flex(Flex { value })
            } else {
                return Err(cssparser::ParseError::custom(()));
            }
        }
        Token::Ident(identifier) => {
            return parse_constant(&identifier);
        }
        Token::ParenthesisBlock => {
            return parse_nested(input, limits, parse_sum);
        }
        Token::Function(name) => {
            return parse_nested(input, limits, |input, limits| {
                parse_function(input, limits, &name)
            });
        }
        _ => return Err(cssparser::ParseError::custom(())),
    };
    Ok(MathExpression::Literal(literal))
}

fn parse_constant(identifier: &str) -> ParseResult<MathExpression> {
    let value = if identifier.eq_ignore_ascii_case("e") {
        std::f32::consts::E
    } else if identifier.eq_ignore_ascii_case("pi") {
        std::f32::consts::PI
    } else if identifier.eq_ignore_ascii_case("infinity") {
        f32::INFINITY
    } else if identifier.eq_ignore_ascii_case("-infinity") {
        f32::NEG_INFINITY
    } else if identifier.eq_ignore_ascii_case("nan") {
        f32::NAN
    } else {
        return Err(cssparser::ParseError::custom(()));
    };
    Ok(MathExpression::Literal(MathLiteral::Number(value)))
}

fn parse_nested<'i, T>(
    input: &mut Parser<'i>,
    limits: &mut ParseLimits,
    parse: impl FnOnce(&mut Parser<'i>, &mut ParseLimits) -> ParseResult<T>,
) -> ParseResult<T> {
    limits.nesting += 1;
    if limits.nesting > MAX_CALC_COMPLEXITY {
        limits.nesting -= 1;
        return Err(cssparser::ParseError::custom(()));
    }
    let result = input.parse_nested_block(|input| parse(input, limits));
    limits.nesting -= 1;
    result
}

fn parse_function<'i>(
    input: &mut Parser<'i>,
    limits: &mut ParseLimits,
    name: &str,
) -> ParseResult<MathExpression> {
    let function = if name.eq_ignore_ascii_case("calc") {
        MathFunction::Calc(parse_sum(input, limits)?)
    } else if name.eq_ignore_ascii_case("min") {
        MathFunction::Min(parse_arguments(input, limits)?)
    } else if name.eq_ignore_ascii_case("max") {
        MathFunction::Max(parse_arguments(input, limits)?)
    } else if name.eq_ignore_ascii_case("clamp") {
        MathFunction::Clamp(parse_clamp(input, limits)?)
    } else if name.eq_ignore_ascii_case("round") {
        MathFunction::Round(parse_round(input, limits)?)
    } else if name.eq_ignore_ascii_case("mod") {
        MathFunction::Mod(parse_pair(input, limits)?)
    } else if name.eq_ignore_ascii_case("rem") {
        MathFunction::Rem(parse_pair(input, limits)?)
    } else if name.eq_ignore_ascii_case("abs") {
        MathFunction::Abs(parse_sum(input, limits)?)
    } else if name.eq_ignore_ascii_case("sign") {
        MathFunction::Sign(parse_sum(input, limits)?)
    } else if name.eq_ignore_ascii_case("hypot") {
        MathFunction::Hypot(parse_arguments(input, limits)?)
    } else if name.eq_ignore_ascii_case("sin") {
        MathFunction::Sin(parse_sum(input, limits)?)
    } else if name.eq_ignore_ascii_case("cos") {
        MathFunction::Cos(parse_sum(input, limits)?)
    } else if name.eq_ignore_ascii_case("tan") {
        MathFunction::Tan(parse_sum(input, limits)?)
    } else if name.eq_ignore_ascii_case("asin") {
        MathFunction::Asin(parse_sum(input, limits)?)
    } else if name.eq_ignore_ascii_case("acos") {
        MathFunction::Acos(parse_sum(input, limits)?)
    } else if name.eq_ignore_ascii_case("atan") {
        MathFunction::Atan(parse_sum(input, limits)?)
    } else if name.eq_ignore_ascii_case("atan2") {
        MathFunction::Atan2(parse_pair(input, limits)?)
    } else if name.eq_ignore_ascii_case("pow") {
        MathFunction::Pow(parse_pair(input, limits)?)
    } else if name.eq_ignore_ascii_case("sqrt") {
        MathFunction::Sqrt(parse_sum(input, limits)?)
    } else if name.eq_ignore_ascii_case("log") {
        MathFunction::Log(parse_log(input, limits)?)
    } else if name.eq_ignore_ascii_case("exp") {
        MathFunction::Exp(parse_sum(input, limits)?)
    } else {
        return Err(cssparser::ParseError::custom(()));
    };
    Ok(MathExpression::Function(Box::new(function)))
}

fn parse_arguments<'i>(
    input: &mut Parser<'i>,
    limits: &mut ParseLimits,
) -> ParseResult<Vec<MathExpression>> {
    let values = input.parse_comma_separated(|input| parse_sum(input, limits))?;
    if values.is_empty() || values.len() > MAX_CALC_COMPLEXITY {
        Err(cssparser::ParseError::custom(()))
    } else {
        Ok(values)
    }
}

fn parse_clamp<'i>(
    input: &mut Parser<'i>,
    limits: &mut ParseLimits,
) -> ParseResult<ClampExpression> {
    let minimum = parse_clamp_bound(input, limits)?;
    input
        .expect_comma()
        .map_err(cssparser::ParseError::<()>::from)?;
    let preferred = input.parse_until_before(cssparser::Delimiter::Comma, |input| {
        parse_sum(input, limits)
    })?;
    input
        .expect_comma()
        .map_err(cssparser::ParseError::<()>::from)?;
    let maximum = parse_clamp_bound(input, limits)?;
    Ok(ClampExpression {
        minimum,
        preferred,
        maximum,
    })
}

fn parse_clamp_bound<'i>(
    input: &mut Parser<'i>,
    limits: &mut ParseLimits,
) -> ParseResult<Option<MathExpression>> {
    if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        Ok(None)
    } else {
        parse_sum(input, limits).map(Some)
    }
}

fn parse_round<'i>(
    input: &mut Parser<'i>,
    limits: &mut ParseLimits,
) -> ParseResult<RoundExpression> {
    let strategy = input
        .try_parse(parse_round_strategy)
        .unwrap_or(RoundingStrategy::Nearest);
    let value = input.parse_until_before(cssparser::Delimiter::Comma, |input| {
        parse_sum(input, limits)
    })?;
    let interval = if input
        .try_parse(|input| {
            input
                .expect_comma()
                .map_err(cssparser::ParseError::<()>::from)
        })
        .is_ok()
    {
        Some(parse_sum(input, limits)?)
    } else {
        None
    };
    Ok(RoundExpression {
        strategy,
        value,
        interval,
    })
}

fn parse_round_strategy<'i>(input: &mut Parser<'i>) -> ParseResult<RoundingStrategy> {
    let identifier = input.expect_ident_cloned()?;
    let strategy = if identifier.eq_ignore_ascii_case("nearest") {
        RoundingStrategy::Nearest
    } else if identifier.eq_ignore_ascii_case("up") {
        RoundingStrategy::Up
    } else if identifier.eq_ignore_ascii_case("down") {
        RoundingStrategy::Down
    } else if identifier.eq_ignore_ascii_case("to-zero") {
        RoundingStrategy::ToZero
    } else {
        return Err(cssparser::ParseError::custom(()));
    };
    input
        .expect_comma()
        .map_err(cssparser::ParseError::<()>::from)?;
    Ok(strategy)
}

fn parse_pair<'i>(input: &mut Parser<'i>, limits: &mut ParseLimits) -> ParseResult<PairExpression> {
    let values = parse_arguments(input, limits)?;
    let [first, second]: [MathExpression; 2] = values
        .try_into()
        .map_err(|_| cssparser::ParseError::custom(()))?;
    Ok(PairExpression { first, second })
}

fn parse_log<'i>(input: &mut Parser<'i>, limits: &mut ParseLimits) -> ParseResult<LogExpression> {
    let values = parse_arguments(input, limits)?;
    match <Vec<MathExpression> as TryInto<[MathExpression; 2]>>::try_into(values) {
        Ok([value, base]) => Ok(LogExpression {
            value,
            base: Some(base),
        }),
        Err(values) => {
            let [value]: [MathExpression; 1] = values
                .try_into()
                .map_err(|_| cssparser::ParseError::custom(()))?;
            Ok(LogExpression { value, base: None })
        }
    }
}

fn binary(operation: BinaryOperation, lhs: MathExpression, rhs: MathExpression) -> MathExpression {
    MathExpression::Binary(Box::new(BinaryExpression {
        operation,
        lhs,
        rhs,
    }))
}

fn parse_length_unit(unit: &str) -> Option<LengthUnit> {
    let unit = if unit.eq_ignore_ascii_case("px") {
        LengthUnit::Absolute(AbsoluteLengthUnit::Px)
    } else if unit.eq_ignore_ascii_case("in") {
        LengthUnit::Absolute(AbsoluteLengthUnit::In)
    } else if unit.eq_ignore_ascii_case("cm") {
        LengthUnit::Absolute(AbsoluteLengthUnit::Cm)
    } else if unit.eq_ignore_ascii_case("mm") {
        LengthUnit::Absolute(AbsoluteLengthUnit::Mm)
    } else if unit.eq_ignore_ascii_case("q") {
        LengthUnit::Absolute(AbsoluteLengthUnit::Q)
    } else if unit.eq_ignore_ascii_case("pt") {
        LengthUnit::Absolute(AbsoluteLengthUnit::Pt)
    } else if unit.eq_ignore_ascii_case("pc") {
        LengthUnit::Absolute(AbsoluteLengthUnit::Pc)
    } else if let Some(unit) = parse_font_unit(unit) {
        LengthUnit::Font(unit)
    } else {
        LengthUnit::Viewport(parse_viewport_unit(unit)?)
    };
    Some(unit)
}

fn parse_font_unit(unit: &str) -> Option<FontLengthUnit> {
    [
        ("em", FontLengthUnit::Em),
        ("rem", FontLengthUnit::Rem),
        ("ex", FontLengthUnit::Ex),
        ("rex", FontLengthUnit::Rex),
        ("ch", FontLengthUnit::Ch),
        ("rch", FontLengthUnit::Rch),
        ("cap", FontLengthUnit::Cap),
        ("rcap", FontLengthUnit::Rcap),
        ("ic", FontLengthUnit::Ic),
        ("ric", FontLengthUnit::Ric),
        ("lh", FontLengthUnit::Lh),
        ("rlh", FontLengthUnit::Rlh),
    ]
    .into_iter()
    .find_map(|(name, parsed)| unit.eq_ignore_ascii_case(name).then_some(parsed))
}

fn parse_viewport_unit(unit: &str) -> Option<ViewportLengthUnit> {
    [
        ("vw", ViewportLengthUnit::Width),
        ("svw", ViewportLengthUnit::SmallWidth),
        ("lvw", ViewportLengthUnit::LargeWidth),
        ("dvw", ViewportLengthUnit::DynamicWidth),
        ("vh", ViewportLengthUnit::Height),
        ("svh", ViewportLengthUnit::SmallHeight),
        ("lvh", ViewportLengthUnit::LargeHeight),
        ("dvh", ViewportLengthUnit::DynamicHeight),
        ("vi", ViewportLengthUnit::Inline),
        ("svi", ViewportLengthUnit::SmallInline),
        ("lvi", ViewportLengthUnit::LargeInline),
        ("dvi", ViewportLengthUnit::DynamicInline),
        ("vb", ViewportLengthUnit::Block),
        ("svb", ViewportLengthUnit::SmallBlock),
        ("lvb", ViewportLengthUnit::LargeBlock),
        ("dvb", ViewportLengthUnit::DynamicBlock),
        ("vmin", ViewportLengthUnit::Min),
        ("svmin", ViewportLengthUnit::SmallMin),
        ("lvmin", ViewportLengthUnit::LargeMin),
        ("dvmin", ViewportLengthUnit::DynamicMin),
        ("vmax", ViewportLengthUnit::Max),
        ("svmax", ViewportLengthUnit::SmallMax),
        ("lvmax", ViewportLengthUnit::LargeMax),
        ("dvmax", ViewportLengthUnit::DynamicMax),
    ]
    .into_iter()
    .find_map(|(name, parsed)| unit.eq_ignore_ascii_case(name).then_some(parsed))
}

fn parse_angle_unit(unit: &str) -> Option<AngleUnit> {
    [
        ("deg", AngleUnit::Deg),
        ("grad", AngleUnit::Grad),
        ("rad", AngleUnit::Rad),
        ("turn", AngleUnit::Turn),
    ]
    .into_iter()
    .find_map(|(name, parsed)| unit.eq_ignore_ascii_case(name).then_some(parsed))
}

fn parse_time_unit(unit: &str) -> Option<TimeUnit> {
    [("s", TimeUnit::Second), ("ms", TimeUnit::Millisecond)]
        .into_iter()
        .find_map(|(name, parsed)| unit.eq_ignore_ascii_case(name).then_some(parsed))
}

fn parse_frequency_unit(unit: &str) -> Option<FrequencyUnit> {
    [
        ("hz", FrequencyUnit::Hertz),
        ("khz", FrequencyUnit::Kilohertz),
    ]
    .into_iter()
    .find_map(|(name, parsed)| unit.eq_ignore_ascii_case(name).then_some(parsed))
}

fn parse_resolution_unit(unit: &str) -> Option<ResolutionUnit> {
    [
        ("dpi", ResolutionUnit::Inch),
        ("dpcm", ResolutionUnit::Centimeter),
        ("dppx", ResolutionUnit::Pixel),
    ]
    .into_iter()
    .find_map(|(name, parsed)| unit.eq_ignore_ascii_case(name).then_some(parsed))
}
