use proc::bind;
use proc::gather;
use proc::scatter;
use proc::then;
use proc::unit;
use proc::yield;

workflow main {
    ret bind(
        scatter([1, 2, 3], (fn(x) { then(yield(), unit(x + 1)) })),
        (fn(handles) { gather(handles) })
    )
}
