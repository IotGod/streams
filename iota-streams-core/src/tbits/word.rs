use std::fmt;

/// Abstraction for a binary/trinary word containing one or several tbits (bits/trits).
/// The size and encoding of the word is defined by the implementation.
/// Many functions take a pair `(d,p)` encoding a slice of tbits as input where
/// `d` is the current tbit offset, `p` is the raw pointer to the first word in a slice.
pub trait BasicTbitWord: Sized + Copy + PartialEq {
    /// The number of tbits per word.
    const SIZE: usize;
    /// Trit or bit.
    type Tbit: Sized + Copy + PartialEq + fmt::Display;

    /// Zero tbit.
    const ZERO_TBIT: Self::Tbit;
    /// All-zero tbits word.
    const ZERO_WORD: Self;

    /// Convert word to SIZE tbits.
    unsafe fn word_to_tbits(x: Self, ts: *mut Self::Tbit);
    /// Convert word from SIZE tbits.
    unsafe fn word_from_tbits(ts: *const Self::Tbit) -> Self;

    unsafe fn put_tbit(d: usize, p: *mut Self, t: Self::Tbit) {
        let mut ts = vec![Self::ZERO_TBIT; Self::SIZE];
        Self::word_to_tbits(*p.add(d / Self::SIZE), ts.as_mut_ptr());
        ts[d % Self::SIZE] = t;
        *p.add(d / Self::SIZE) = Self::word_from_tbits(ts.as_ptr());
    }
    unsafe fn get_tbit(d: usize, p: *const Self) -> Self::Tbit {
        let mut ts = vec![Self::ZERO_TBIT; Self::SIZE];
        Self::word_to_tbits(*p.add(d / Self::SIZE), ts.as_mut_ptr());
        ts[d % Self::SIZE]
    }

    unsafe fn fold_tbits<F>(n: usize, dx: usize, x: *const Self, mut f: F)
    where
        F: FnMut(&[Self::Tbit]),
    {
        if n == 0 {
            return;
        }

        // Primitive array `[Self::Tbit; Self::SIZE]` is not supported yet.
        let mut v = vec![Self::ZERO_TBIT; Self::SIZE];
        let rx = dx % Self::SIZE;
        let mut xx = x.add(dx / Self::SIZE);
        let mut nn = n;
        let mut d;

        if rx != 0 {
            d = std::cmp::min(n, Self::SIZE - rx);
            Self::word_to_tbits(*xx, v.as_mut_ptr());
            f(&v[rx..rx + d]);
            nn -= d;
            xx = xx.add(1);
        }

        d = Self::SIZE;
        while nn >= d {
            Self::word_to_tbits(*xx, v.as_mut_ptr());
            f(&v[..]);
            nn -= d;
            xx = xx.add(1);
        }

        if nn > 0 {
            Self::word_to_tbits(*xx, v.as_mut_ptr());
            f(&v[..nn]);
        }
    }

    unsafe fn refold_tbits<F>(n: usize, dx: usize, x: *mut Self, mut f: F)
    where
        F: FnMut(&mut [Self::Tbit]),
    {
        if n == 0 {
            return;
        }

        // Primitive array `[Self::Tbit; Self::SIZE]` is not supported yet.
        let mut v = vec![Self::ZERO_TBIT; Self::SIZE];
        let rx = dx % Self::SIZE;
        let mut xx = x.add(dx / Self::SIZE);
        let mut nn = n;
        let mut d;

        if rx != 0 {
            d = std::cmp::min(n, Self::SIZE - rx);
            Self::word_to_tbits(*xx, v.as_mut_ptr());
            f(&mut v[rx..rx + d]);
            *xx = Self::word_from_tbits(v.as_ptr());
            nn -= d;
            xx = xx.add(1);
        }

        d = Self::SIZE;
        while nn >= d {
            Self::word_to_tbits(*xx, v.as_mut_ptr());
            f(&mut v[..]);
            *xx = Self::word_from_tbits(v.as_ptr());
            nn -= d;
            xx = xx.add(1);
        }

        if nn > 0 {
            Self::word_to_tbits(*xx, v.as_mut_ptr());
            f(&mut v[..nn]);
            *xx = Self::word_from_tbits(v.as_ptr());
        }
    }

    unsafe fn unfold_tbits<F>(n: usize, dx: usize, x: *mut Self, mut f: F)
    where
        F: FnMut(&mut [Self::Tbit]),
    {
        if n == 0 {
            return;
        }

        // Primitive array `[Self::Tbit; Self::SIZE]` is not supported yet.
        let mut v = vec![Self::ZERO_TBIT; Self::SIZE];
        let rx = dx % Self::SIZE;
        let mut xx = x.add(dx / Self::SIZE);
        let mut nn = n;
        let mut d;

        if rx != 0 {
            d = std::cmp::min(n, Self::SIZE - rx);
            Self::word_to_tbits(*xx, v.as_mut_ptr());
            f(&mut v[rx..rx + d]);
            *xx = Self::word_from_tbits(v.as_ptr());
            nn -= d;
            xx = xx.add(1);
        }

        d = Self::SIZE;
        while nn >= d {
            f(&mut v[..]);
            *xx = Self::word_from_tbits(v.as_ptr());
            nn -= d;
            xx = xx.add(1);
        }

        if nn > 0 {
            Self::word_to_tbits(*xx, v.as_mut_ptr());
            f(&mut v[..nn]);
            *xx = Self::word_from_tbits(v.as_ptr());
        }
    }

    unsafe fn to_tbits(n: usize, dx: usize, x: *const Self, mut ts: *mut Self::Tbit) {
        Self::fold_tbits(n, dx, x, |tx| {
            std::ptr::copy(tx.as_ptr(), ts, tx.len());
            ts = ts.add(tx.len());
        });
    }

    unsafe fn from_tbits(n: usize, dx: usize, x: *mut Self, mut ts: *const Self::Tbit) {
        Self::unfold_tbits(n, dx, x, |tx| {
            std::ptr::copy(ts, tx.as_mut_ptr(), tx.len());
            ts = ts.add(tx.len());
        });
    }

    /// Copy `n` tbits from `(dx,x)` slice into `(dy,y)`.
    unsafe fn copy(n: usize, dx: usize, x: *const Self, dy: usize, y: *mut Self) {
        if n == 0 {
            return;
        }

        let rx = dx % Self::SIZE;
        let mut xx = x.add(dx / Self::SIZE);
        let ry = dy % Self::SIZE;
        let mut yy = y.add(dy / Self::SIZE);
        let mut nn = n;

        if rx == ry {
            let mut xs = vec![Self::ZERO_TBIT; Self::SIZE];
            let mut ys = vec![Self::ZERO_TBIT; Self::SIZE];

            if rx != 0 {
                Self::word_to_tbits(*xx, xs.as_mut_ptr());
                Self::word_to_tbits(*yy, ys.as_mut_ptr());
                let d = std::cmp::min(n, Self::SIZE - rx);
                ys[ry..ry + d].copy_from_slice(&xs[rx..rx + d]);
                *yy = Self::word_from_tbits(ys.as_ptr());

                nn -= d;
                xx = xx.add(1);
                yy = yy.add(1);
            }

            std::ptr::copy(xx, yy, nn / Self::SIZE);

            xx = xx.add(nn / Self::SIZE);
            yy = yy.add(nn / Self::SIZE);
            nn = nn % Self::SIZE;

            if nn != 0 {
                Self::word_to_tbits(*xx, xs.as_mut_ptr());
                Self::word_to_tbits(*yy, ys.as_mut_ptr());
                ys[0..nn].copy_from_slice(&xs[0..nn]);
                *yy = Self::word_from_tbits(ys.as_ptr());
            }
        } else {
            // Rare case, just convert via tbits.
            let mut ts = vec![Self::ZERO_TBIT; n];
            Self::to_tbits(n, dx, x, ts.as_mut_ptr());
            Self::from_tbits(n, dy, y, ts.as_ptr());
        }
    }

    /// Set `n` tbits in `(dx,x)` slice to zero.
    unsafe fn set_zero(n: usize, dx: usize, x: *mut Self) {
        if n == 0 {
            return;
        }

        let mut v = vec![Self::ZERO_TBIT; Self::SIZE];
        let rx = dx % Self::SIZE;
        let mut xx = x.add(dx / Self::SIZE);
        let mut nn = n;
        let mut d;

        if rx != 0 {
            d = std::cmp::min(n, Self::SIZE - rx);
            Self::word_to_tbits(*xx, v.as_mut_ptr());
            for i in rx..rx + d {
                *v.as_mut_ptr().add(i) = Self::ZERO_TBIT;
            }
            *xx = Self::word_from_tbits(v.as_ptr());
            nn -= d;
            xx = xx.add(1);
        }

        d = Self::SIZE;
        while nn >= d {
            *xx = Self::ZERO_WORD;
            nn -= d;
            xx = xx.add(1);
        }

        if nn > 0 {
            Self::word_to_tbits(*xx, v.as_mut_ptr());
            for i in 0..nn {
                *v.as_mut_ptr().add(i) = Self::ZERO_TBIT;
            }
            *xx = Self::word_from_tbits(v.as_ptr());
        }
    }

    /// Compare `n` tbits from `(dx,x)` slice into `(dy,y)`.
    unsafe fn equals(n: usize, dx: usize, x: *const Self, dy: usize, y: *const Self) -> bool {
        if n == 0 {
            return true;
        }

        let rx = dx % Self::SIZE;
        let mut xx = x.add(dx / Self::SIZE);
        let ry = dy % Self::SIZE;
        let mut yy = y.add(dy / Self::SIZE);
        let mut nn = n;

        if rx == ry {
            let mut xs = vec![Self::ZERO_TBIT; Self::SIZE];
            let mut ys = vec![Self::ZERO_TBIT; Self::SIZE];

            if rx != 0 {
                Self::word_to_tbits(*xx, xs.as_mut_ptr());
                Self::word_to_tbits(*yy, ys.as_mut_ptr());
                let d = std::cmp::min(n, Self::SIZE - rx);
                if ys[ry..ry + d] != xs[rx..rx + d] {
                    return false;
                }

                nn -= d;
                xx = xx.add(1);
                yy = yy.add(1);
            }

            while nn >= Self::SIZE {
                if *xx != *yy {
                    return false;
                }
                nn -= Self::SIZE;
                xx = xx.add(1);
                yy = yy.add(1);
            }

            xx = xx.add(nn / Self::SIZE);
            yy = yy.add(nn / Self::SIZE);
            nn = nn % Self::SIZE;

            if nn != 0 {
                Self::word_to_tbits(*xx, xs.as_mut_ptr());
                Self::word_to_tbits(*yy, ys.as_mut_ptr());
                if ys[0..nn] != xs[0..nn] {
                    return false;
                }
            }

            true
        } else {
            // Rare case, just convert via tbits.
            let mut xs = vec![Self::ZERO_TBIT; n];
            let mut ys = vec![Self::ZERO_TBIT; n];
            Self::to_tbits(n, dx, x, xs.as_mut_ptr());
            Self::to_tbits(n, dy, y, ys.as_mut_ptr());
            xs == ys
        }
    }
}

pub trait StringTbitWord: BasicTbitWord {
    const TBITS_PER_CHAR: usize;
    unsafe fn put_char(s: usize, d: usize, p: *mut Self, c: char) -> bool;
    unsafe fn get_char(s: usize, d: usize, p: *const Self) -> char;
}

pub trait IntTbitWord: BasicTbitWord {
    unsafe fn put_isize(n: usize, d: usize, p: *mut Self, i: isize);
    unsafe fn get_isize(n: usize, d: usize, p: *const Self) -> isize;
    unsafe fn put_usize(n: usize, d: usize, p: *mut Self, u: usize);
    unsafe fn get_usize(n: usize, d: usize, p: *const Self) -> usize;
}

pub trait SpongosTbitWord: BasicTbitWord {
    // Spongos-related utils

    /// x+y
    fn tbit_add(x: Self::Tbit, y: Self::Tbit) -> Self::Tbit;
    /// x-y
    fn tbit_sub(x: Self::Tbit, y: Self::Tbit) -> Self::Tbit;

    /// s:=s+x
    unsafe fn add(mut ds: usize, s: *mut Self, n: usize, mut dx: usize, x: *const Self) {
        for _ in 0..n {
            let ts = Self::get_tbit(ds, s);
            let tx = Self::get_tbit(dx, x);
            let ty = Self::tbit_add(ts, tx);
            Self::put_tbit(ds, s, ty);
            dx += 1;
            ds += 1;
        }
    }

    /// y:=x+s, s:=x, x:=y
    unsafe fn setx_add_mut(mut ds: usize, s: *mut Self, n: usize, mut dx: usize, x: *mut Self) {
        for _ in 0..n {
            let ts = Self::get_tbit(ds, s);
            let tx = Self::get_tbit(dx, x);
            let ty = Self::tbit_add(tx, ts);
            Self::put_tbit(ds, s, tx);
            Self::put_tbit(dx, x, ty);
            dx += 1;
            ds += 1;
        }
    }
    /// x:=y-s, s:=x, y:=x
    unsafe fn setx_sub_mut(mut ds: usize, s: *mut Self, n: usize, mut dy: usize, y: *mut Self) {
        for _ in 0..n {
            let ts = Self::get_tbit(ds, s);
            let ty = Self::get_tbit(dy, y);
            let tx = Self::tbit_sub(ty, ts);
            Self::put_tbit(ds, s, tx);
            Self::put_tbit(dy, y, tx);
            dy += 1;
            ds += 1;
        }
    }
    /// y:=x+s, s:=x
    unsafe fn setx_add(
        mut ds: usize,
        s: *mut Self,
        n: usize,
        mut dx: usize,
        x: *const Self,
        mut dy: usize,
        y: *mut Self,
    ) {
        for _ in 0..n {
            let ts = Self::get_tbit(ds, s);
            let tx = Self::get_tbit(dx, x);
            let ty = Self::tbit_add(tx, ts);
            Self::put_tbit(ds, s, tx);
            Self::put_tbit(dy, y, ty);
            dx += 1;
            ds += 1;
            dy += 1;
        }
    }
    /// x:=y-s, s:=x
    unsafe fn setx_sub(
        mut ds: usize,
        s: *mut Self,
        n: usize,
        mut dy: usize,
        y: *const Self,
        mut dx: usize,
        x: *mut Self,
    ) {
        for _ in 0..n {
            let ts = Self::get_tbit(ds, s);
            let ty = Self::get_tbit(dy, y);
            let tx = Self::tbit_sub(ty, ts);
            Self::put_tbit(ds, s, tx);
            Self::put_tbit(dx, x, tx);
            dx += 1;
            ds += 1;
            dy += 1;
        }
    }

    /// y:=x+s, s:=y, x:=y
    unsafe fn sety_add_mut(mut ds: usize, s: *mut Self, n: usize, mut dx: usize, x: *mut Self) {
        for _ in 0..n {
            let ts = Self::get_tbit(ds, s);
            let tx = Self::get_tbit(dx, x);
            let ty = Self::tbit_add(tx, ts);
            Self::put_tbit(ds, s, ty);
            Self::put_tbit(dx, x, ty);
            dx += 1;
            ds += 1;
        }
    }
    /// x:=y-s, s:=y, y:=x
    unsafe fn sety_sub_mut(mut ds: usize, s: *mut Self, n: usize, mut dy: usize, y: *mut Self) {
        for _ in 0..n {
            let ts = Self::get_tbit(ds, s);
            let ty = Self::get_tbit(dy, y);
            let tx = Self::tbit_sub(ty, ts);
            Self::put_tbit(ds, s, ty);
            Self::put_tbit(dy, y, tx);
            dy += 1;
            ds += 1;
        }
    }
    /// y:=x+s, s:=y
    unsafe fn sety_add(
        mut ds: usize,
        s: *mut Self,
        n: usize,
        mut dx: usize,
        x: *const Self,
        mut dy: usize,
        y: *mut Self,
    ) {
        for _ in 0..n {
            let tx = Self::get_tbit(dx, x);
            let ts = Self::get_tbit(ds, s);
            let ty = Self::tbit_add(tx, ts);
            Self::put_tbit(ds, s, ty);
            Self::put_tbit(dy, y, ty);
            dx += 1;
            ds += 1;
            dy += 1;
        }
    }
    /// x:=y-s, s:=y
    unsafe fn sety_sub(
        mut ds: usize,
        s: *mut Self,
        n: usize,
        mut dy: usize,
        y: *const Self,
        mut dx: usize,
        x: *mut Self,
    ) {
        for _ in 0..n {
            let ty = Self::get_tbit(dy, y);
            let ts = Self::get_tbit(ds, s);
            let tx = Self::tbit_sub(ty, ts);
            Self::put_tbit(ds, s, ty);
            Self::put_tbit(dx, x, tx);
            dx += 1;
            ds += 1;
            dy += 1;
        }
    }

    /// Absorb plain tbits `x` into state `s`, OVERWRITE mode.
    unsafe fn absorb_overwrite(ds: usize, s: *mut Self, n: usize, dx: usize, x: *const Self) {
        Self::copy(n, dx, x, ds, s);
    }
    /// Absorb plain tbits `x` into state `s`, ADD/XOR mode.
    unsafe fn absorb_xor(ds: usize, s: *mut Self, n: usize, dx: usize, x: *const Self) {
        Self::add(ds, s, n, dx, x);
    }

    /// Squeeze tbits `y` from state `s`, OVERWRITE mode.
    unsafe fn squeeze_overwrite(ds: usize, s: *mut Self, n: usize, dy: usize, y: *mut Self) {
        Self::copy(n, ds, s, dy, y);
        Self::set_zero(n, ds, s);
    }
    /// Squeeze tbits `y` from state `s`, ADD/XOR mode.
    unsafe fn squeeze_xor(ds: usize, s: *mut Self, n: usize, dy: usize, y: *mut Self) {
        Self::copy(n, ds, s, dy, y);
    }

    /// Squeeze tbits `y` from state `s`, OVERWRITE mode.
    unsafe fn squeeze_eq_overwrite(
        ds: usize,
        s: *mut Self,
        n: usize,
        dy: usize,
        y: *const Self,
    ) -> bool {
        let r = Self::equals(n, ds, s as *const Self, dy, y);
        Self::set_zero(n, ds, s);
        r
    }
    /// Squeeze tbits `y` from state `s`, ADD/XOR mode.
    unsafe fn squeeze_eq_xor(ds: usize, s: *mut Self, n: usize, dy: usize, y: *const Self) -> bool {
        Self::equals(n, ds, s as *const Self, dy, y)
    }

    /// Encrypt tbits `x` into `y` with state `s`, OVERWRITE mode.
    unsafe fn encrypt_overwrite(
        ds: usize,
        s: *mut Self,
        n: usize,
        dx: usize,
        x: *const Self,
        dy: usize,
        y: *mut Self,
    ) {
        Self::setx_add(ds, s, n, dx, x, dy, y);
    }
    /// Encrypt tbits `x` with state `s`, OVERWRITE mode.
    unsafe fn encrypt_overwrite_mut(ds: usize, s: *mut Self, n: usize, dx: usize, x: *mut Self) {
        Self::setx_add_mut(ds, s, n, dx, x);
    }
    /// Encrypt tbits `y` with state `s`, ADD/XOR mode.
    unsafe fn encrypt_xor(
        ds: usize,
        s: *mut Self,
        n: usize,
        dx: usize,
        x: *const Self,
        dy: usize,
        y: *mut Self,
    ) {
        Self::sety_add(ds, s, n, dx, x, dy, y);
    }
    /// Encrypt tbits `x` with state `s`, ADD/XOR mode.
    unsafe fn encrypt_xor_mut(ds: usize, s: *mut Self, n: usize, dx: usize, x: *mut Self) {
        Self::sety_add_mut(ds, s, n, dx, x);
    }

    /// Decrypt tbits `y` into `x` with state `s`, OVERWRITE mode.
    unsafe fn decrypt_overwrite(
        ds: usize,
        s: *mut Self,
        n: usize,
        dy: usize,
        y: *const Self,
        dx: usize,
        x: *mut Self,
    ) {
        Self::setx_sub(ds, s, n, dy, y, dx, x);
    }
    /// Decrypt tbits `y` with state `s`, OVERWRITE mode.
    unsafe fn decrypt_overwrite_mut(ds: usize, s: *mut Self, n: usize, dy: usize, y: *mut Self) {
        Self::setx_sub_mut(ds, s, n, dy, y);
    }
    /// Decrypt tbits `y` into `x` with state `s`, ADD/XOR mode.
    unsafe fn decrypt_xor(
        ds: usize,
        s: *mut Self,
        n: usize,
        dy: usize,
        y: *const Self,
        dx: usize,
        x: *mut Self,
    ) {
        Self::sety_sub(ds, s, n, dy, y, dx, x);
    }
    /// Decrypt tbits `y` with state `s`, ADD/XOR mode.
    unsafe fn decrypt_xor_mut(ds: usize, s: *mut Self, n: usize, dy: usize, y: *mut Self) {
        Self::sety_sub_mut(ds, s, n, dy, y);
    }
}
