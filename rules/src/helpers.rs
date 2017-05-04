use rustling::*;
use dimension::*;
use moment::*;
use std::ops;

pub fn compose_numbers(a: &NumberValue, b: &NumberValue) -> RuleResult<NumberValue> {
    let grain = a.grain().unwrap_or(0) as u32;
    if 10u64.pow(grain) as f32 > b.value() {
        match (a, b) {
            (&NumberValue::Integer(ref lhs), &NumberValue::Integer(ref rhs)) => {
                Ok(NumberValue::Integer(IntegerValue::new(lhs.value + rhs.value)?))
            }
            _ => Ok(NumberValue::Float(FloatValue::new(a.value() + b.value())?)),
        }
    } else {
        Err(RuleErrorKind::Invalid.into())
    }
}

pub fn compose_money(a: &AmountOfMoneyValue,
                     b: &AmountOfMoneyValue)
                     -> RuleResult<AmountOfMoneyValue> {
    let amount = a.value + b.value / 100.0;
    Ok(AmountOfMoneyValue {
           value: amount,
           unit: a.unit,
           ..AmountOfMoneyValue::default()
       })
}

pub fn compose_money_number(a: &AmountOfMoneyValue,
                            b: &NumberValue)
                            -> RuleResult<AmountOfMoneyValue> {
    let amount = a.value + b.value() / 100.0;
    Ok(AmountOfMoneyValue {
           value: amount,
           unit: a.unit,
           ..AmountOfMoneyValue::default()
       })
}

impl Form {
    fn time_of_day(full_hour: u32, is_12_clock: bool) -> Form {
        Form::TimeOfDay(Some(TimeOfDayForm { full_hour: full_hour, is_12_clock: is_12_clock }))
    }
}

impl TimeValue {
    pub fn constraint(constraint: RcConstraint) -> TimeValue {
        TimeValue {
            constraint: constraint,
            form: Form::Empty,
            direction: None,
        }
    }

    pub fn form(self, form: Form) -> TimeValue {
        TimeValue {
            form: form,
            .. self 
        }
    }

    pub fn direction(self, direction: Option<Direction>) -> TimeValue {
        TimeValue {
            direction: direction,
            .. self
        }
    }

    pub fn intersect(self, other: TimeValue) -> RuleResult<TimeValue> {
        Ok(TimeValue::constraint(self.constraint.intersect(other.constraint))
            .direction(self.direction.or(other.direction)))
    }

    pub fn last_of(self, other: TimeValue) -> RuleResult<TimeValue> {
        Ok(TimeValue::constraint(self.constraint.last_of(other.constraint)))
    }

    pub fn the_nth(self, n: i64) -> RuleResult<TimeValue> {
        Ok(TimeValue::constraint(self.constraint.take_the_nth(n)))
    }

    pub fn the_nth_not_immediate(self, n: i64) -> RuleResult<TimeValue> {
        Ok(TimeValue::constraint(self.constraint.take_the_nth_not_immediate(n)))
    }

    pub fn the_nth_after(self, n: i64, after_value: TimeValue) -> RuleResult<TimeValue> {
        Ok(TimeValue::constraint(self.constraint.the_nth(n).after_not_immediate(after_value.constraint)))
    }

    pub fn span_to(self, to: TimeValue, is_inclusive: bool) -> RuleResult<TimeValue> {
        if (self.constraint.grain() == Grain::Day && to.constraint.grain() == Grain::Day) || is_inclusive {
            Ok(TimeValue::constraint(self.constraint.span_inclusive_to(to.constraint)))
        } else {
            Ok(TimeValue::constraint(self.constraint.span_to(to.constraint)))
        }
    }
}

pub fn year(y: i32) -> RuleResult<TimeValue> {
    Ok(TimeValue::constraint(Year::new(y)))
}

pub fn month(m: u32) -> RuleResult<TimeValue> {
    if !(1 <= m && m <= 12) { unimplemented!();  } 
    Ok(TimeValue::constraint(Month::new(m)).form(Form::Month(m)))
}

pub fn day_of_month(dom: u32) -> RuleResult<TimeValue> {
    if !(1 <= dom && dom <= 31) { unimplemented!(); }
    Ok(TimeValue::constraint(DayOfMonth::new(dom)).form(Form::Empty))
}

pub fn month_day(m: u32, d: u32) -> RuleResult<TimeValue> {
    Ok(month(m)?.intersect(day_of_month(d)?)?)
}

pub fn hour(h: u32, is_12_clock: bool) -> RuleResult<TimeValue> {
    if is_12_clock {
        Ok(TimeValue::constraint(Hour::clock_12(h)).form(Form::time_of_day(h, true)))
    } else {
        Ok(TimeValue::constraint(Hour::clock_24(h)).form(Form::time_of_day(h, true)))
    }
}

pub fn minute(m: u32) -> RuleResult<TimeValue> {
    Ok(TimeValue::constraint(Minute::new(m)))
}

pub fn second(s: u32) -> RuleResult<TimeValue> {
    Ok(TimeValue::constraint(Second::new(s)))
}

pub fn hour_minute(h: u32, m: u32, is_12_clock: bool) -> RuleResult<TimeValue> {
    Ok(hour(h, is_12_clock)?.intersect(minute(m)?)?.form(Form::TimeOfDay(None)))
}

pub fn hour_minute_second_clock_12(h: u32, m: u32, s: u32, is_12_clock: bool) -> RuleResult<TimeValue> {
    Ok(hour_minute(h, m, is_12_clock)?.intersect(second(s)?)?.form(Form::TimeOfDay(None)))
}

pub fn hour_minute_relative(h: u32, m: i32, is_12_clock: bool) -> RuleResult<TimeValue> {
    if !(1 <= h && h <= 23) { unimplemented!(); }
    if !(-59 <= m && m <= 59) { unimplemented!(); }
    let normalized_minute = ((m + 60) % 60) as u32;

    let shifter_hour = if m >= 0 {
        h
    } else {
        match (h, is_12_clock) {
            (0, true) => 23,
            (1, true) => 12,
            (0, false) => 23,
            (1, false) => 0,
            _ => h - 1
        }
    };
    hour_minute(shifter_hour, normalized_minute, is_12_clock)
}

pub fn cycle(grain: Grain) -> RuleResult<TimeValue> {
    Ok(TimeValue::constraint(Cycle::rc(grain)))
}

pub fn cycle_nth(grain: Grain, n: i64) -> RuleResult<TimeValue> {
    Ok(TimeValue::constraint(Cycle::rc(grain).take_the_nth(n)))
}

pub fn cycle_nth_after(grain: Grain, n: i64, after_value: TimeValue) -> RuleResult<TimeValue> {
    Ok(TimeValue::constraint(Cycle::rc(grain).the_nth(n).after(after_value.constraint)))
}

pub fn cycle_nth_after_not_immediate(grain: Grain, n: i64, after_value: TimeValue) -> RuleResult<TimeValue> {
    Ok(TimeValue::constraint(Cycle::rc(grain).the_nth(n).after_not_immediate(after_value.constraint)))
}

pub fn cycle_n(grain: Grain, n: i64) -> RuleResult<TimeValue> {
    Ok(TimeValue::constraint(Cycle::rc(grain).take(n)))
}

pub fn cycle_n_not_immediate(grain: Grain, n: i64) -> RuleResult<TimeValue> {
    Ok(TimeValue::constraint(Cycle::rc(grain).take_not_immediate(n)))
}

pub fn ymd(y: i32, m: u32, d: u32) -> RuleResult<TimeValue> {
    Ok(year(y)?.intersect(month_day(m, d)?)?)
}

#[derive(Clone)]
struct DurationValue(Period);

impl DurationValue {
    fn new(grain: Grain, n: i64) -> DurationValue {
        DurationValue(PeriodComp::new(grain, n).into())
    }

    pub fn in_present(self) -> RuleResult<TimeValue> {
        Ok(TimeValue::constraint(Cycle::rc(Grain::Second).take_the_nth(0).shift_by(self.0)))
    }

    pub fn ago(self) -> RuleResult<TimeValue> {
        Ok(TimeValue::constraint(Cycle::rc(Grain::Second).take_the_nth(0).shift_by(-self.0)))
    } 

    pub fn after(self, time: TimeValue) -> RuleResult<TimeValue> {
        Ok(TimeValue::constraint(time.constraint.shift_by(self.0)))
    }

    pub fn before(self, time: TimeValue) -> RuleResult<TimeValue> {
        Ok(TimeValue::constraint(time.constraint.shift_by(-self.0)))
    }
}

impl ops::Add<DurationValue> for DurationValue {
    type Output = DurationValue;
    fn add(self, duration: DurationValue) -> DurationValue {
        DurationValue(self.0 + duration.0)
    }
}

impl<'a> ops::Add<&'a DurationValue> for DurationValue {
    type Output = DurationValue;
    fn add(self, duration: &'a DurationValue) -> DurationValue {
        DurationValue(self.0 + &duration.0)
    }
}

impl<'a, 'b> ops::Add<&'a DurationValue> for &'b DurationValue {
    type Output = DurationValue;
    fn add(self, duration: &'a DurationValue) -> DurationValue {
        DurationValue(&self.0 + &duration.0)
    }
}

impl<'a> ops::Add<DurationValue> for &'a DurationValue {
    type Output = DurationValue;
    fn add(self, duration: DurationValue) -> DurationValue {
        DurationValue(&self.0 + duration.0)
    }
}
