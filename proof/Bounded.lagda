\documentclass{article}

\usepackage{agda}
\usepackage{agdon}
\usepackage{bytefield}

\hypersetup{
            pdftitle={Specification and (nearly complete) Proofs for Registers with Bounded Types},
            pdfborder={0 0 0},
            breaklinks=true}

\title{Specification and (nearly complete) Proofs for Registers with Bounded Types}

\author{Dan Pittman dan@auxon.io}

\begin{document}

\maketitle

\begin{abstract}
A literate Agda file containing a specification-as-types as well as
proofs of the bounds properties of the registers library.
\end{abstract}

\section{Introduction}

Register access and manipulation exists at the lowest levels of a
software stack. due to its foundational nature, its correctness should
be considered to be of the utmost importance. When software interacts
with registers on the hardware, it does so through the manipulation of
data referred to by \emph{a priori} specified pointers. It is common
practice to treat the data at these pointers as unsigned integers
whose width is determined by the register's width.

We then naturally arrive at a bounded range of positive integers which
can be represented given the number of bits available. Denotationally,
we can think of these unsigned integers as the natural numbers less
than or equal to some upper bound.

\begin{code}[hide]
module Bounded where

open import Data.Nat hiding (_≥_)
open import Data.Nat.Properties
open import Relation.Binary.PropositionalEquality hiding (trans)

data _≥_ : ℕ → ℕ → Set where
  n≥z : {y : ℕ} → y ≥ zero
  s≥s : {x y : ℕ} → x ≥ y → (suc x) ≥ (suc y)
\end{code}

\section{Ranges}

When we think about ranges logically, we'd normally consider them as
the set:

$$
\text{Range}(l, u) := \{x\ |\ l\ \leqslant\ x\ \leqslant\ u\}
$$

However, when working within the confines of Rust's type system, it is
easier to consider a range like so:

$$
\text{Range}(l, u) := \{x\ |\ x\ \geqslant\ l\ \land\ x\ \leqslant\ u\}
$$

The following type represents the statement above. Its constructor
requires a proof that $x \geqslant l$ as well as a proof that $x \leq
u$. From those proofs, we build a new type, \AgdaDatatype{InRange},
parameterized by its lower bound, the value, and its upper bound.

\begin{code}
data InRange (l u : ℕ) : Set where
  in-range : (x : ℕ) → (x ≥ l) → (x ≤ u) → InRange l u

get-rv : ∀ {l u} → InRange l u → ℕ
get-rv (in-range x _ _ ) = x
\end{code}

\section{Registers}

As stated in the introduction, we intend to enforce some properties
about interacting with registers. Typically, a register is made of up
fields, which occupy some range of bits present in that register. When
reading, writing, or modifying a register, the preferred API is one
which delineates the fields such that the programmer can consider the
fields' \emph{logical} values, rather than the value relative to their
position in the register. In the following section, we will cover the
logical representation of these register fields.

\subsection{Fields}

For simplicity's sake, we will illustrate with an 8-bit register with
three fields, \texttt{On}, \texttt{Dead}, and \texttt{Color}.

$$
\begin{bytefield}[endianness=little,bitwidth=0.1\linewidth]{8}
\bitheader{0-7} \\
\bitbox{1}{\texttt{On}} &
\bitbox{1}{\texttt{Dead}} &
\bitbox{3}{\texttt{Color}} &
\bitbox{3}{Unused}
\end{bytefield}
$$

When we interact with these fields, we'd like to do so with their
logical values—$0$ or $1$ for \texttt{On} and \texttt{Dead}, and the
range $0...7$ for \texttt{Color}—as opposed to their values relative
to their position in the register. Because of this expectation, we are
exposed to the possiblity of invoking a field's API with a value which
exceeds its upper bound. Therefore, we'd like for the API we expose to
disallow such a possibility altogether, and therefore preventing
undefined behavior from rearing its ugly head.

To compute a field's range, we take its width in the register in which
it resides and use it to compute the maximum logical value. This is a
simple base-two shifting operation:

$$
(1 << \text{width}) - 1
$$

We'll need to define our own \AgdaFunction{<<}. Once we do, we can
define a type which uses \AgdaDatatype{InRange} to represent a field
in a register.

\begin{code}[hide]
--|This is just a dirty implementation of subtraction for ℕ. Please
--|note that if it reaches zero on the lhs, it returns 0. This would
--|cause a range to have an upper bound of zero, with which we would
--|not be able to prove an inductive case. I state this here to
--|assuage any worries about the overall soundness.
_-_ : ℕ → ℕ → ℕ
0 - y = 0
(suc x) - (suc y) = x - y
{-# CATCHALL #-}
x - 0 = x
\end{code}

\begin{code}
_<<_ : ℕ → ℕ → ℕ
0 << _ = 0
{-# CATCHALL #-}
x << (suc y) = (x * 2) << y
{-# CATCHALL #-}
x << 0 = x

width-max : ℕ → ℕ
width-max w = ((1 << w) - 1)

data Field : (w o : ℕ) → Set where
  mk-field : {w o : ℕ} → InRange 0 (width-max w) → Field w o

field-mask : ∀ {w o} → Field w o → ℕ
field-mask {w} {o} _ = (width-max w) << o
\end{code}

Now, because \AgdaDatatype{Field} requires an \AgdaDatatype{InRange}
proof, we can think it merely a refinement to the more general case
we've proven above. Let's prove an example of a field with our
register above.

We begin with a proof that the third color, whose
\texttt{Color}-relevant value is \texttt{2}, does indeed fit into the
\texttt{Color} field.

\begin{code}
color-can-be-two : 2 ≤ (width-max 3)
color-can-be-two = s≤s (s≤s z≤n)
\end{code}

Then, we can use that proof when a constructing a \texttt{Color} field
whose value is \texttt{2}.

\begin{code}
color-is-two : Field 3 2
color-is-two = mk-field (in-range 2 n≥z color-can-be-two)
\end{code}


\section{A Register is the sum of its parts}

We begin with a binary representation of $\mathbb{N}$, where the least
significant bit is first. Through this definition, we can do bitwise
operations on the registers and their fields.
\begin{code}
data Bin : Set where
  end : Bin
  zero : Bin → Bin
  one : Bin → Bin

inc-bin : Bin → Bin
inc-bin end = one end
inc-bin (one b) = zero (inc-bin b)
inc-bin (zero b) = one b

nat→bin : ℕ → Bin
nat→bin 0 = end
nat→bin (suc n) = inc-bin (nat→bin n)

bin→nat : Bin → ℕ
bin→nat end = 0
bin→nat (zero b) = 2 * bin→nat b
bin→nat (one b) = 1 + 2 * (bin→nat b)
\end{code}

Next, we prove that our $\mathbb{N} \rightarrow \text{Binary}
\rightarrow \mathbb{N}$ conversion abides the initially given
$\mathbb{N}$ for all $\mathbb{N}$.

\begin{code}
lemma-suc-inc : ∀ b → bin→nat (inc-bin b) ≡ suc (bin→nat b)
lemma-suc-inc end = refl
lemma-suc-inc (zero b) = refl
lemma-suc-inc (one b) =
  begin
    bin→nat (inc-bin b) + (bin→nat (inc-bin b) + 0)
  ≡⟨ cong (λ x → ( x + (bin→nat (inc-bin b) + 0))) (lemma-suc-inc b) ⟩
    suc (bin→nat b) + (bin→nat (inc-bin b) + 0)
  ≡⟨ cong (λ x → ( suc (bin→nat b) + (x + 0))) (lemma-suc-inc b) ⟩
    suc (bin→nat b) + (suc (bin→nat b) + 0)
  ≡⟨ cong suc (+-suc (bin→nat b) ((bin→nat b) + 0)) ⟩
    suc (suc (bin→nat b + (bin→nat b + 0)))
  ∎ where open ≡-Reasoning

nat→bin→nat : ∀ n → bin→nat (nat→bin n) ≡ n
nat→bin→nat zero = refl
nat→bin→nat (suc n) =
  begin
    bin→nat (nat→bin (suc n))
  ≡⟨⟩
    bin→nat (inc-bin (nat→bin n))
  ≡⟨ lemma-suc-inc (nat→bin n) ⟩
    suc (bin→nat (nat→bin n))
  ≡⟨ cong suc (nat→bin→nat n) ⟩
    suc n
  ∎ where open ≡-Reasoning

_&b_ : Bin → Bin → Bin
_ &b end = end
(one x) &b (one y) = one (x &b y)
(zero x) &b (one y) = zero (x &b y)
(one x) &b (zero y) = zero (x &b y)
(zero x) &b (zero y) = zero (x &b y)
{-# CATCHALL #-}
end &b _ = end

_&_ : ℕ → ℕ → ℕ
x & y = bin→nat ((nat→bin x) &b (nat→bin y))

_>>_ : ℕ → ℕ → ℕ
0 >> _ = 0
(suc x) >> (suc y) = ⌊ (suc x) /2⌋ >> y
{-# CATCHALL #-}
(suc x) >> 0 = (suc x)
\end{code}

We know that a \AgdaDatatype{Field} carries with it a proof that its
value resides within its bounds. When we consider a register as merely
the composite of its fields, we, through construction, have a safe API
to the register itself, because our only interaction with the register
is through its fields. This can be demonstrated by asserting that the
summation of the widths of the fields said to be contained in a
register is $\leqslant$ the total width of the register.

\begin{code}
data Register : (w : ℕ) → Set where
  end : ∀ {w} → Register w
  with-field : ∀ {w fw fo : ℕ} →
               Field fw fo →
               fw + fo ≤ w →
               Register w →
               Register (w - fw)
\end{code}

Here, the construction of a register is an inductive family where each
field added to it deducts from the available width for the register.
With this as our type for a register, we know with certitude:

\begin{enumerate}

\item Only fields which fit in a register can be said to reside within
      that register, that's the $fw + fo \leqslant w$ part.

\item The constraints put on a field by this definition allow us to
      prove that interaction with fields never contravene their
      bounds. We demonstrate that with a proof of reading any
      arbitrary field in any register below in
      \AgdaFunction{read-prf}.

\end{enumerate}

The operation $(\text{RegVal } \&\ \text{FieldMask}) >>
\text{FieldOffset}$ tells us how to read a field from a
register. Before we can get to a proof regarding this operation, we
first must get some lemmas out of the way.

\begin{code}[hide]
--| Here are a few uninteresting proofs needed to build up to our
--| final result.
&-zero : ∀ n → n & 0 ≡ 0
&-zero zero = refl
&-zero (suc n) = refl

shl-zero : ∀ n → 0 << n ≡ 0
shl-zero n = refl 

shr-zero : ∀ n → 0 >> n ≡ 0
shr-zero n = refl

width-max-zero : (width-max 0) ≡ 0
width-max-zero  = refl

postulate &b-≤ : ∀ b c → ((bin→nat b) & (bin→nat c)) ≤ (bin→nat c)
postulate >>-≤ : ∀ {n m w} → (n ≤ m) → (n >> w) ≤ (m >> w)
postulate suc-<<>>-cancel : ∀ n m → ((suc n) << m) >> m ≡ suc ((n << m) >> m)
\end{code}

First, we prove that \texttt{\&}-ing any value produces a value less
than or equal to that value.

\begin{code}
&-≤ : ∀ n m → (n & m) ≤ m
&-≤ n zero rewrite &-zero n = z≤n
&-≤ n (suc m) = begin
  n & (suc m) ≡⟨ sym (cong (λ x → (x & (suc m))) (nat→bin→nat n)) ⟩
  (bin→nat (nat→bin n)) & (suc m)
    ≡⟨ sym (cong (λ x → ((bin→nat (nat→bin n)) & x)) (nat→bin→nat (suc m))) ⟩
  (bin→nat (nat→bin n)) & (bin→nat (nat→bin (suc m)))
    ≤⟨ &b-≤ (nat→bin n) (nat→bin (suc m)) ⟩
  bin→nat (nat→bin (suc m)) ≡⟨ nat→bin→nat (suc m) ⟩
  (suc m)
  ∎ where open ≤-Reasoning
\end{code}

Next, we prove that a right or left shift by zero preserves the value
on the left-hand side.

\begin{code}
<<-identity : ∀ n → n << 0 ≡ n
<<-identity 0 = refl
<<-identity (suc n) = refl

>>-identity : ∀ n → n >> 0 ≡ n
>>-identity 0 = refl
>>-identity (suc n) = refl
\end{code}

Now, a proof that shifting a value left then right by the same value
yields the initial value.

\begin{code}
<<>>-cancel : ∀ n m → (n << m) >> m ≡ n
<<>>-cancel 0 m rewrite shl-zero m | shr-zero m = refl
<<>>-cancel (suc n) m rewrite suc-<<>>-cancel n m = begin
  suc ((n << m) >> m) ≡⟨ cong suc (<<>>-cancel n m) ⟩
  suc n
  ∎ where open ≡-Reasoning
\end{code}

Now, finally, we discharge each of those lemmas in a proof that register
reads abide their bounds.

\begin{code}
read-prf : ∀ fw rv fo → (rv & ((width-max fw) << fo)) >> fo ≤ (width-max fw)
read-prf zero rv fo rewrite width-max-zero | shl-zero fo | &-zero rv | shr-zero fo = z≤n
read-prf (suc fw) rv fo = begin
  (rv & ((width-max (suc fw)) << fo)) >> fo
    ≤⟨ >>-≤ (&-≤ rv ((width-max (suc fw) << fo))) ⟩
  ((width-max (suc fw)) << fo) >> fo ≡⟨ <<>>-cancel (width-max (suc fw)) fo ⟩
  width-max (suc fw)
  ∎ where open ≤-Reasoning
\end{code}
\end{document}
