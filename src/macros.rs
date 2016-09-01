macro_rules! range {
    ($e: expr, $f: expr, $t: expr, $c:ty) => ($e as $c >= $f as $c && $e as $c <= $t as $c);
}
