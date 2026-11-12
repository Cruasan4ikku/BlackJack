use rand::seq::SliceRandom;
use rand::thread_rng;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

impl Suit {
    pub fn symbol(self) -> char {
        match self {
            Self::Clubs => '♣',
            Self::Diamonds => '♦',
            Self::Hearts => '♥',
            Self::Spades => '♠',
        }
    }

    pub fn is_red(self) -> bool {
        matches!(self, Self::Diamonds | Self::Hearts)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rank {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl Rank {
    pub fn value(self) -> u8 {
        match self {
            Self::Two => 2,
            Self::Three => 3,
            Self::Four => 4,
            Self::Five => 5,
            Self::Six => 6,
            Self::Seven => 7,
            Self::Eight => 8,
            Self::Nine => 9,
            Self::Ten | Self::Jack | Self::Queen | Self::King => 10,
            Self::Ace => 11,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Two => "2",
            Self::Three => "3",
            Self::Four => "4",
            Self::Five => "5",
            Self::Six => "6",
            Self::Seven => "7",
            Self::Eight => "8",
            Self::Nine => "9",
            Self::Ten => "10",
            Self::Jack => "J",
            Self::Queen => "Q",
            Self::King => "K",
            Self::Ace => "A",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Card {
    pub suit: Suit,
    pub rank: Rank,
}

impl std::fmt::Display for Card {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}{}", self.rank.label(), self.suit.symbol())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Hand {
    pub cards: Vec<Card>,
}

impl Hand {
    pub fn score(&self) -> u8 {
        let mut total: u8 = self.cards.iter().map(|card| card.rank.value()).sum();
        let mut aces = self.cards.iter().filter(|card| card.rank == Rank::Ace).count();

        while total > 21 && aces > 0 {
            total -= 10;
            aces -= 1;
        }

        total
    }

    pub fn is_blackjack(&self) -> bool {
        self.cards.len() == 2 && self.score() == 21
    }

    pub fn is_busted(&self) -> bool {
        self.score() > 21
    }

    fn add(&mut self, card: Card) {
        self.cards.push(card);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameState {
    Menu,
    PlayerTurn,
    DealerTurn,
    RoundOver,
}

pub struct Game {
    pub state: GameState,
    pub bankroll: u32,
    pub bet: u32,
    pub player_hands: Vec<Hand>,
    pub hand_bets: Vec<u32>,
    pub active_hand: usize,
    pub dealer: Hand,
    pub dealer_hole_card_hidden: bool,
    pub message: String,
    deck: Vec<Card>,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            state: GameState::Menu,
            bankroll: 1_000,
            bet: 100,
            player_hands: Vec::new(),
            hand_bets: Vec::new(),
            active_hand: 0,
            dealer: Hand::default(),
            dealer_hole_card_hidden: true,
            message: "Place your bet and deal.".to_owned(),
            deck: Vec::new(),
        }
    }
}

impl Game {
    pub fn set_bet(&mut self, bet: u32) {
        if self.state == GameState::Menu || self.state == GameState::RoundOver {
            self.bet = bet.clamp(1, self.bankroll.max(1));
        }
    }

    pub fn deal(&mut self) {
        if !matches!(self.state, GameState::Menu | GameState::RoundOver) {
            return;
        }
        if self.bankroll == 0 {
            self.message = "You are out of chips. Restart the game to play again.".to_owned();
            return;
        }

        self.bet = self.bet.clamp(1, self.bankroll);
        self.bankroll -= self.bet;
        self.player_hands = vec![Hand::default()];
        self.hand_bets = vec![self.bet];
        self.active_hand = 0;
        self.dealer = Hand::default();
        self.dealer_hole_card_hidden = true;
        self.deck = shuffled_deck();
        let player_card = self.draw_card();
        self.player_hands[0].add(player_card);
        let dealer_card = self.draw_card();
        self.dealer.add(dealer_card);
        let player_card = self.draw_card();
        self.player_hands[0].add(player_card);
        let dealer_card = self.draw_card();
        self.dealer.add(dealer_card);

        let player_blackjack = self.player_hands[0].is_blackjack();
        let dealer_blackjack = self.dealer.is_blackjack();
        if player_blackjack || dealer_blackjack {
            self.finish_initial_blackjack(player_blackjack, dealer_blackjack);
        } else {
            self.state = GameState::PlayerTurn;
            self.message = "Your turn.".to_owned();
        }
    }

    pub fn player_hit(&mut self) {
        if self.state != GameState::PlayerTurn {
            return;
        }
        let card = self.draw_card();
        self.player_hands[self.active_hand].add(card);
        if self.player_hands[self.active_hand].is_busted() {
            self.message = format!("Hand {} busted.", self.active_hand + 1);
            self.advance_hand();
        } else {
            self.message = "Your turn.".to_owned();
        }
    }

    pub fn player_stand(&mut self) {
        if self.state != GameState::PlayerTurn {
            return;
        }
        self.advance_hand();
    }

    pub fn player_double(&mut self) {
        if self.state != GameState::PlayerTurn
            || self.player_hands[self.active_hand].cards.len() != 2
            || self.bankroll < self.hand_bets[self.active_hand]
        {
            return;
        }
        let additional_bet = self.hand_bets[self.active_hand];
        self.bankroll -= additional_bet;
        self.hand_bets[self.active_hand] += additional_bet;
        let card = self.draw_card();
        self.player_hands[self.active_hand].add(card);
        if self.player_hands[self.active_hand].is_busted() {
            self.message = format!("Hand {} busted.", self.active_hand + 1);
        }
        self.advance_hand();
    }

    pub fn player_split(&mut self) {
        if self.state != GameState::PlayerTurn
            || self.player_hands[self.active_hand].cards.len() != 2
            || self.bankroll < self.hand_bets[self.active_hand]
        {
            return;
        }

        let hand_index = self.active_hand;
        let first = self.player_hands[hand_index].cards[0];
        let second = self.player_hands[hand_index].cards[1];
        if first.rank != second.rank {
            return;
        }

        let split_bet = self.hand_bets[hand_index];
        self.bankroll -= split_bet;
        let first_draw = self.draw_card();
        let second_draw = self.draw_card();
        self.player_hands[hand_index].cards = vec![first, first_draw];
        self.player_hands.insert(hand_index + 1, Hand { cards: vec![second, second_draw] });
        self.hand_bets.insert(hand_index + 1, split_bet);
        self.message = format!("Playing hand {} of {}.", hand_index + 1, self.player_hands.len());
    }

    pub fn can_split(&self) -> bool {
        self.state == GameState::PlayerTurn
            && self.bankroll >= self.hand_bets[self.active_hand]
            && self.player_hands[self.active_hand].cards.len() == 2
            && self.player_hands[self.active_hand].cards[0].rank
                == self.player_hands[self.active_hand].cards[1].rank
    }

    pub fn can_double(&self) -> bool {
        self.state == GameState::PlayerTurn
            && self.bankroll >= self.hand_bets[self.active_hand]
            && self.player_hands[self.active_hand].cards.len() == 2
    }

    fn finish_initial_blackjack(&mut self, player_blackjack: bool, dealer_blackjack: bool) {
        self.state = GameState::RoundOver;
        self.dealer_hole_card_hidden = false;
        match (player_blackjack, dealer_blackjack) {
            (true, true) => {
                self.bankroll += self.bet;
                self.message = "Push — both have Blackjack.".to_owned();
            }
            (true, false) => {
                self.bankroll += self.bet + self.bet * 3 / 2;
                self.message = "Blackjack! You win 3:2.".to_owned();
            }
            (false, true) => self.message = "Dealer has Blackjack.".to_owned(),
            (false, false) => unreachable!(),
        }
    }

    fn advance_hand(&mut self) {
        if self.active_hand + 1 < self.player_hands.len() {
            self.active_hand += 1;
            self.message = format!("Playing hand {} of {}.", self.active_hand + 1, self.player_hands.len());
        } else {
            self.play_dealer_turn();
        }
    }

    fn play_dealer_turn(&mut self) {
        self.state = GameState::DealerTurn;
        self.dealer_hole_card_hidden = false;
        if self.player_hands.iter().all(Hand::is_busted) {
            self.finish_round();
            return;
        }
        while self.dealer.score() < 17 {
            let card = self.draw_card();
            self.dealer.add(card);
        }
        self.finish_round();
    }

    fn finish_round(&mut self) {
        let dealer_score = self.dealer.score();
        let dealer_busted = self.dealer.is_busted();
        let mut wins = 0;
        let mut pushes = 0;
        let mut losses = 0;
        for (hand, bet) in self.player_hands.iter().zip(self.hand_bets.iter()) {
            if hand.is_busted() {
                losses += 1;
            } else if dealer_busted || hand.score() > dealer_score {
                self.bankroll += bet * 2;
                wins += 1;
            } else if hand.score() == dealer_score {
                self.bankroll += bet;
                pushes += 1;
            } else {
                losses += 1;
            }
        }
        self.state = GameState::RoundOver;
        self.message = match (wins, pushes, losses) {
            (w, 0, 0) => format!("You win {} hand(s)!", w),
            (0, p, 0) => format!("Push on {} hand(s).", p),
            (0, 0, l) => format!("Dealer wins {} hand(s).", l),
            _ => format!("Round complete: {} won, {} pushed, {} lost.", wins, pushes, losses),
        };
    }

    fn draw_card(&mut self) -> Card {
        if self.deck.is_empty() {
            self.deck = shuffled_deck();
        }
        self.deck.pop().expect("a shuffled deck contains cards")
    }
}

fn shuffled_deck() -> Vec<Card> {
    let suits = [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades];
    let ranks = [
        Rank::Two, Rank::Three, Rank::Four, Rank::Five, Rank::Six, Rank::Seven, Rank::Eight,
        Rank::Nine, Rank::Ten, Rank::Jack, Rank::Queen, Rank::King, Rank::Ace,
    ];
    let mut deck = suits
        .iter()
        .flat_map(|suit| ranks.iter().map(move |rank| Card { suit: *suit, rank: *rank }))
        .collect::<Vec<_>>();
    deck.shuffle(&mut thread_rng());
    deck
}

#[cfg(test)]
mod tests {
    use super::*;

    fn card(rank: Rank) -> Card {
        Card { suit: Suit::Spades, rank }
    }

    #[test]
    fn aces_count_as_eleven_or_one_to_avoid_bust() {
        let hand = Hand { cards: vec![card(Rank::Ace), card(Rank::Nine), card(Rank::Five)] };
        assert_eq!(hand.score(), 15);
        assert!(!hand.is_busted());
    }

    #[test]
    fn blackjack_requires_two_cards() {
        let blackjack = Hand { cards: vec![card(Rank::Ace), card(Rank::King)] };
        let three_card_21 = Hand { cards: vec![card(Rank::Seven), card(Rank::Seven), card(Rank::Seven)] };
        assert!(blackjack.is_blackjack());
        assert!(!three_card_21.is_blackjack());
    }

    #[test]
    fn bet_is_capped_at_available_bankroll() {
        let mut game = Game::default();
        game.set_bet(5_000);
        assert_eq!(game.bet, game.bankroll);
    }
}
