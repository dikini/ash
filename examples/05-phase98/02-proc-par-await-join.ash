use proc::await;
use proc::bind;
use proc::join;
use proc::par;
use proc::then;
use proc::unit;
use proc::yield;

workflow main {
    ret bind(
        par(
            then(yield(), unit(41)),
            then(yield(), unit(1))
        ),
        (fn(await_handles) {
            bind(
                par(
                    then(yield(), unit(41)),
                    then(yield(), unit(1))
                ),
                (fn(join_handles) {
                    unit(record(
                        "await_handles", await_handles,
                        "await_observer",
                        bind(
                            await(await_handles.0),
                            (fn(left) {
                                bind(
                                    await(await_handles.1),
                                    (fn(right) {
                                        unit(record("left", left, "right", right))
                                    })
                                )
                            })
                        ),
                        "join_handles", join_handles,
                        "join_observer", join(join_handles.0, join_handles.1)
                    ))
                })
            )
        })
    )
}
