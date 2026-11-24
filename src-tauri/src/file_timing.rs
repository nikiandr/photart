use std::cell::RefCell;
use std::time::{Duration, Instant};

pub(crate) struct FileTiming {
    started: Instant,
    stages: RefCell<Vec<(&'static str, Duration)>>,
}

impl FileTiming {
    pub(crate) fn start() -> Option<Self> {
        cfg!(debug_assertions).then(|| Self {
            started: Instant::now(),
            stages: RefCell::new(Vec::new()),
        })
    }

    pub(crate) fn print_header() {
        eprintln!(
            "  {:>4}  {:<5}  {:>7} {:>7} {:>7} {:>7} {:>7} {:>7}  Result",
            "File", "Type", "Total", "Read", "Wait", "Preview", "Write", "Pair"
        );
    }

    pub(crate) fn report(&self, file: usize, format: &str, outcome: &str, verbose: bool) {
        let stages = self.stages.borrow();
        let mut totals = [None; 5];
        for &(stage, elapsed) in stages.iter() {
            let column = match stage {
                "source read" => 0,
                "decoder lock wait" => 1,
                "thumbnail write" | "staging directory" | "original write" => 3,
                "pairing metadata" => 4,
                _ => 2,
            };
            *totals[column].get_or_insert(Duration::ZERO) += elapsed;
        }
        let [read, wait, preview, write, pair] = totals.map(|elapsed| {
            elapsed.map_or_else(
                || "-".to_owned(),
                |elapsed| format!("{:.3}", elapsed.as_secs_f64()),
            )
        });
        let total = self.started.elapsed().as_secs_f64();
        let row = format!(
            "  {file:>4}  {format:<5.5}  {total:>7.3} {read:>7} {wait:>7} {preview:>7} {write:>7} {pair:>7}  {outcome}"
        );
        if verbose {
            let details = stages
                .iter()
                .map(|(stage, elapsed)| format!("{stage} {:.2?}", elapsed))
                .collect::<Vec<_>>()
                .join(" · ");
            eprintln!("{row}\n        {details}");
        } else {
            eprintln!("{row}");
        }
    }
}

pub(crate) fn measure<T>(
    timing: Option<&FileTiming>,
    stage: &'static str,
    work: impl FnOnce() -> T,
) -> T {
    let started = timing.map(|_| Instant::now());
    let result = work();
    if let Some((timing, started)) = timing.zip(started) {
        timing.stages.borrow_mut().push((stage, started.elapsed()));
    }
    result
}
