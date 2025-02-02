//! Some custom assertion types.

pub use tracing::error;

/// Assert in debug mode, log error in release mode.
#[macro_export]
macro_rules! log_assert {
    ($cond:expr $(,)?) => {
        let cond = $cond;
        ::std::debug_assert!(cond);
        if (!cond) {
            $crate::assertions::error!(
                "INVARIANT VIOLATED in {}:{}: {}",
                ::std::file!(),
                ::std::line!(),
                ::std::stringify!($cond),
            );
        }
    };
    ($cond:expr, $($fmt:expr),+ $(,)?) => {
        let cond = $cond;
        ::std::debug_assert!(cond, $($fmt,)+);
        if (!cond) {
            $crate::assertions::error!(
                "INVARIANT VIOLATED in {}:{}: {}",
                ::std::file!(),
                ::std::line!(),
                ::std::format_args!($($fmt,)+),
            );
        }
    };
}

/// Assert equality in debug mode, log error in release mode.
#[macro_export]
macro_rules! log_assert_eq {
    ($lhs:expr, $rhs:expr $(,)?) => {
        match (&$lhs, &$rhs) {
            (lhs, rhs) => {
                ::std::assert_eq!(lhs, rhs);
                if !(*lhs == *rhs) {
                    $crate::assertions::error!(
                        "INVARIANT VIOLATED in {}:{}: left: {lhs:?}, right: {rhs:?}",
                        ::std::file!(),
                        ::std::line!(),
                    );
                }
            }
        }
    };
    ($lhs:expr, $rhs:expr, $($fmt:expr),+ $(,)?) => {
        match (&$lhs, &$rhs) {
            (lhs, rhs) => {
                ::std::assert_eq!(lhs, rhs, $($fmt,)+);
                if !(*lhs == *rhs) {
                    $crate::assertions::error!(
                        "INVARIANT VIOLATED in {}:{}: left: {lhs:?}, right: {rhs:?}, {}",
                        ::std::file!(),
                        ::std::line!(),
                        ::std::format_args!($($fmt,)+),
                    );
                }
            }
        }
    };
}

#[cfg(test)]
mod test {
    #[test]
    fn log_assert_ok() {
        let x = 5;
        log_assert!(true);
        log_assert!(true,);
        log_assert!(true, "foo");
        log_assert!(true, "foo",);
        log_assert!(true, "foo{}", x);
        log_assert!(true, "foo{}", x + 1,);
        log_assert!(true, "foo{x}");
    }

    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn log_assert_fail() {
        log_assert!(false);
    }

    #[test]
    fn log_assert_eq_ok() {
        let x = 5;
        log_assert_eq!(x, 5);
        log_assert_eq!(x, 5, "foo");
        log_assert_eq!(x, 5, "foo",);
        log_assert_eq!(x, 5, "foo{}", x + 3);
        log_assert_eq!(x, 5, "foo{}", x + 4,);
        log_assert_eq!(x, 5, "foo{x}");
    }

    #[test]
    fn log_assert_eq_should_borrow() {
        let x = String::from("hello");
        log_assert_eq!(x, "hello");
        // This would fail to compile if the macro "consumed" `x`.
        drop(x);
    }

    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn log_assert_eq_fail() {
        let x = 5;
        log_assert_eq!(x, 6);
    }

    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn log_assert_eq_msg() {
        let x = 5;
        log_assert_eq!(x, 6, "5 and 6 not the same");
    }

    #[test]
    fn eval_once() {
        // Check side effects are evaluated only once inside the macros.
        let mut a = 0u32;

        log_assert!({
            a += 1;
            true
        });
        log_assert!(
            {
                a += 10;
                true
            },
            "foo"
        );
        log_assert_eq!(
            {
                a += 100;
                5
            },
            5
        );
        log_assert_eq!(
            {
                a += 1000;
                5
            },
            5,
            "baz"
        );
        log_assert_eq!(6, {
            a += 10000;
            6
        });
        log_assert_eq!(
            7,
            {
                a += 100000;
                7
            },
            "bar"
        );

        assert_eq!(a, 111111);
    }
}
