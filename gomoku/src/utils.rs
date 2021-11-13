use std::{
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
  time::{Duration, Instant},
};

pub fn do_run(end: &Arc<AtomicBool>) -> bool {
  !end.load(Ordering::Relaxed)
}

pub fn print_status(msg: &str, end_time: &Instant) {
  println!(
    "{} ({:?} remaining)",
    msg,
    (*end_time)
      .checked_duration_since(Instant::now())
      .unwrap_or(Duration::ZERO)
  );
}

#[allow(
  clippy::cast_precision_loss,
  clippy::cast_possible_truncation,
  clippy::cast_sign_loss
)]
pub fn format_number(number: f32) -> String {
  let sizes = [' ', 'k', 'M', 'G', 'T'];

  let base = 1000.0;
  let i = number.log(base).floor();
  let number = format!("{:.2}", number / base.powi(i as i32));
  if i > 1.0 {
    format!("{}{}", number, sizes[i as usize])
  } else {
    number
  }
}
