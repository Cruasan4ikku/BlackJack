mod game;

use eframe::egui;
use game::{Game, GameState};

const BACKGROUND: egui::Color32 = egui::Color32::from_rgb(10, 22, 24);
const FELT: egui::Color32 = egui::Color32::from_rgb(17, 91, 67);
const FELT_DARK: egui::Color32 = egui::Color32::from_rgb(12, 72, 53);
const GOLD: egui::Color32 = egui::Color32::from_rgb(226, 190, 105);
const MUTED: egui::Color32 = egui::Color32::from_rgb(180, 199, 190);
const CARD_WIDTH: f32 = 72.0;
const CARD_HEIGHT: f32 = 102.0;
const CARD_STAGGER: f64 = 0.12;
const CARD_TRAVEL_TIME: f64 = 0.32;

#[derive(Clone, Copy)]
struct CardAnimation {
    started_at: f64,
    first_card: usize,
    card_count: usize,
}

struct BlackjackApp {
    game: Game,
    card_animation: Option<CardAnimation>,
}

impl Default for BlackjackApp {
    fn default() -> Self {
        Self {
            game: Game::default(),
            card_animation: None,
        }
    }
}

impl eframe::App for BlackjackApp {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        let now = context.input(|input| input.time);
        let mut animation = self.card_animation;

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(BACKGROUND))
            .show(context, |ui| {
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.heading(
                            egui::RichText::new("BLACKJACK")
                                .size(30.0)
                                .strong()
                                .color(GOLD),
                        );
                        ui.label(
                            egui::RichText::new("CLASSIC TABLE")
                                .size(11.0)
                                .strong()
                                .color(MUTED),
                        );
                    });

                    let badges_width = 306.0;
                    ui.add_space((ui.available_width() - badges_width).max(12.0));
                    info_pill(ui, "BET", format!("${}", self.game.bet));
                    ui.add_space(10.0);
                    info_pill(ui, "BALANCE", format!("${}", self.game.bankroll));
                });

                if matches!(self.game.state, GameState::Menu | GameState::RoundOver) {
                    ui.add_space(14.0);
                    egui::Frame::none()
                        .fill(FELT_DARK)
                        .rounding(egui::Rounding::same(12.0))
                        .inner_margin(egui::Margin::symmetric(14.0, 10.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("PLACE YOUR BET")
                                        .size(12.0)
                                        .strong()
                                        .color(GOLD),
                                );
                                ui.add_space(8.0);
                                if ui.button("- $25").clicked() {
                                    self.game.set_bet(self.game.bet.saturating_sub(25).max(1));
                                }
                                let mut bet = self.game.bet as i32;
                                if ui
                                    .add(
                                        egui::DragValue::new(&mut bet)
                                            .range(1..=self.game.bankroll.max(1) as i32)
                                            .speed(10.0)
                                            .prefix("$"),
                                    )
                                    .changed()
                                {
                                    self.game.set_bet(bet.max(1) as u32);
                                }
                                if ui.button("+ $25").clicked() {
                                    self.game.set_bet(
                                        self.game.bet.saturating_add(25).min(self.game.bankroll),
                                    );
                                }
                                ui.add_space(8.0);
                                if action_button(
                                    ui,
                                    if self.game.state == GameState::Menu {
                                        "DEAL"
                                    } else {
                                        "NEW ROUND"
                                    },
                                    true,
                                )
                                .clicked()
                                {
                                    self.game.deal();
                                    animation = Some(CardAnimation {
                                        started_at: now,
                                        first_card: 0,
                                        card_count: count_cards(&self.game),
                                    });
                                }
                            });
                        });
                }

                ui.add_space(18.0);
                egui::Frame::none()
                    .fill(FELT)
                    .stroke(egui::Stroke::new(2.0_f32, GOLD))
                    .rounding(egui::Rounding::same(22.0))
                    .inner_margin(egui::Margin::same(20.0))
                    .show(ui, |ui| {
                        ui.set_min_height(410.0);

                        ui.horizontal(|ui| {
                            section_title(ui, "DEALER");
                            ui.add_space(12.0);
                            if !self.game.dealer.cards.is_empty() {
                                let dealer_score = if self.game.dealer_hole_card_hidden {
                                    format!("UPCARD {}", self.game.dealer.cards[0].rank.value())
                                } else {
                                    format!("SCORE {}", self.game.dealer.score())
                                };
                                score_badge(ui, &dealer_score);
                            }
                        });

                        let mut card_index = 0;
                        render_cards(
                            ui,
                            &self.game.dealer.cards,
                            self.game.dealer_hole_card_hidden,
                            &mut card_index,
                            animation,
                            now,
                        );

                        ui.add_space(22.0);
                        ui.separator();
                        ui.add_space(14.0);

                        ui.horizontal(|ui| {
                            section_title(ui, "PLAYER");
                            if !self.game.player_hands.is_empty() {
                                ui.add_space(12.0);
                                score_badge(
                                    ui,
                                    &format!(
                                        "{} HAND{}",
                                        self.game.player_hands.len(),
                                        if self.game.player_hands.len() == 1 { "" } else { "S" }
                                    ),
                                );
                            }
                        });

                        for (hand_index, hand) in self.game.player_hands.iter().enumerate() {
                            let active = self.game.state == GameState::PlayerTurn
                                && hand_index == self.game.active_hand;
                            ui.horizontal(|ui| {
                                let hand_name = if active {
                                    format!("HAND {}  /  YOUR TURN", hand_index + 1)
                                } else {
                                    format!("HAND {}", hand_index + 1)
                                };
                                ui.label(
                                    egui::RichText::new(hand_name)
                                        .size(12.0)
                                        .strong()
                                        .color(if active { GOLD } else { MUTED }),
                                );
                                ui.add_space(8.0);
                                score_badge(
                                    ui,
                                    &format!(
                                        "SCORE {}   /   BET ${}",
                                        hand.score(),
                                        self.game.hand_bets[hand_index]
                                    ),
                                );
                                if hand.is_busted() {
                                    ui.colored_label(
                                        egui::Color32::from_rgb(255, 130, 120),
                                        "BUST",
                                    );
                                } else if hand.is_blackjack() {
                                    ui.colored_label(GOLD, "BLACKJACK");
                                }
                            });

                            render_cards(
                                ui,
                                &hand.cards,
                                false,
                                &mut card_index,
                                animation,
                                now,
                            );
                            ui.add_space(10.0);
                        }
                    });

                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new(&self.game.message)
                                .size(15.0)
                                .strong()
                                .color(GOLD),
                        );
                    });
                    ui.with_layout(
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| match self.game.state {
                            GameState::PlayerTurn => {
                                if action_button(ui, "STAND", false).clicked() {
                                    let before = count_cards(&self.game);
                                    self.game.player_stand();
                                    animation =
                                        animation_for_change(&self.game, before, before, now);
                                }
                                if action_button(ui, "HIT", true).clicked() {
                                    let before = count_cards(&self.game);
                                    self.game.player_hit();
                                    animation = animation_for_change(
                                        &self.game,
                                        before,
                                        before,
                                        now,
                                    );
                                }
                                if ui
                                    .add_enabled(
                                        self.game.can_split(),
                                        egui::Button::new(
                                            egui::RichText::new("SPLIT").strong(),
                                        )
                                        .fill(FELT_DARK)
                                        .stroke(egui::Stroke::new(1.0_f32, GOLD))
                                        .min_size(egui::vec2(86.0, 40.0)),
                                    )
                                    .clicked()
                                {
                                    let before = count_cards(&self.game);
                                    self.game.player_split();
                                    animation = Some(CardAnimation {
                                        started_at: now,
                                        first_card: 2,
                                        card_count: count_cards(&self.game).saturating_sub(2),
                                    });
                                    if count_cards(&self.game) <= before {
                                        animation = None;
                                    }
                                }
                                if ui
                                    .add_enabled(
                                        self.game.can_double(),
                                        egui::Button::new(
                                            egui::RichText::new("DOUBLE").strong(),
                                        )
                                        .fill(FELT_DARK)
                                        .stroke(egui::Stroke::new(1.0_f32, GOLD))
                                        .min_size(egui::vec2(92.0, 40.0)),
                                    )
                                    .clicked()
                                {
                                    let before = count_cards(&self.game);
                                    self.game.player_double();
                                    animation = animation_for_change(
                                        &self.game,
                                        before,
                                        before,
                                        now,
                                    );
                                }
                            }
                            GameState::Menu | GameState::RoundOver => {}
                            GameState::DealerTurn => {}
                        },
                    );
                });
            });

        if let Some(active_animation) = animation {
            let duration = active_animation.card_count as f64 * CARD_STAGGER
                + CARD_TRAVEL_TIME;
            if now - active_animation.started_at < duration {
                context.request_repaint();
            } else {
                animation = None;
            }
        }
        self.card_animation = animation;
    }
}

fn info_pill(ui: &mut egui::Ui, title: &str, value: String) {
    egui::Frame::none()
        .fill(FELT_DARK)
        .rounding(egui::Rounding::same(8.0))
        .inner_margin(egui::Margin::symmetric(12.0, 8.0))
        .show(ui, |ui| {
            ui.set_min_width(124.0);
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new(title)
                        .size(10.0)
                        .strong()
                        .color(MUTED),
                );
                ui.label(
                    egui::RichText::new(value)
                        .size(18.0)
                        .strong()
                        .color(GOLD),
                );
            });
        });
}

fn section_title(ui: &mut egui::Ui, title: &str) {
    ui.label(
        egui::RichText::new(title)
            .size(17.0)
            .strong()
            .color(egui::Color32::WHITE),
    );
}

fn score_badge(ui: &mut egui::Ui, text: &str) {
    egui::Frame::none()
        .fill(FELT_DARK)
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(egui::Margin::symmetric(10.0, 5.0))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(text)
                    .size(11.0)
                    .strong()
                    .color(GOLD),
            );
        });
}

fn action_button(ui: &mut egui::Ui, title: &str, primary: bool) -> egui::Response {
    let fill = if primary { GOLD } else { FELT_DARK };
    let text_color = if primary { BACKGROUND } else { egui::Color32::WHITE };
    ui.add_sized(
        [100.0, 42.0],
        egui::Button::new(egui::RichText::new(title).strong().color(text_color))
            .fill(fill)
            .stroke(egui::Stroke::new(1.0_f32, GOLD)),
    )
}

fn count_cards(game: &Game) -> usize {
    game.dealer.cards.len()
        + game
            .player_hands
            .iter()
            .map(|hand| hand.cards.len())
            .sum::<usize>()
}

fn animation_for_change(
    game: &Game,
    previous_count: usize,
    first_card: usize,
    now: f64,
) -> Option<CardAnimation> {
    let added = count_cards(game).saturating_sub(previous_count);
    if added == 0 {
        None
    } else {
        Some(CardAnimation {
            started_at: now,
            first_card,
            card_count: added,
        })
    }
}

fn render_cards(
    ui: &mut egui::Ui,
    cards: &[game::Card],
    hide_hole_card: bool,
    card_index: &mut usize,
    animation: Option<CardAnimation>,
    now: f64,
) {
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        for (index, card) in cards.iter().enumerate() {
            let hidden = hide_hole_card && index == 1;
            render_card(ui, Some(*card), hidden, *card_index, animation, now);
            *card_index += 1;
        }
    });
}

fn render_card(
    ui: &mut egui::Ui,
    card: Option<game::Card>,
    hidden: bool,
    index: usize,
    animation: Option<CardAnimation>,
    now: f64,
) {
    let (slot, _) = ui.allocate_exact_size(
        egui::vec2(CARD_WIDTH, CARD_HEIGHT),
        egui::Sense::hover(),
    );
    let progress = animation_progress(index, animation, now);
    let source = ui.ctx().input(|input| input.screen_rect().center())
        + egui::vec2(0.0, 100.0);
    let rect = slot.translate((source - slot.center()) * (1.0 - progress));
    let painter = ui.painter();

    if hidden {
        painter.rect_filled(rect, 8.0, egui::Color32::from_rgb(31, 57, 91));
        painter.rect_stroke(
            rect.shrink(5.0),
            5.0,
            egui::Stroke::new(1.5_f32, GOLD),
        );
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "♠",
            egui::FontId::proportional(34.0),
            egui::Color32::from_rgb(202, 217, 240),
        );
        return;
    }

    let Some(card) = card else {
        return;
    };
    let ink = if card.suit.is_red() {
        egui::Color32::from_rgb(190, 45, 54)
    } else {
        egui::Color32::from_rgb(25, 34, 39)
    };

    painter.rect_filled(rect.translate(egui::vec2(2.0, 3.0)), 8.0, FELT_DARK);
    painter.rect_filled(rect, 8.0, egui::Color32::from_rgb(249, 247, 239));
    painter.rect_stroke(
        rect,
        8.0,
        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(220, 216, 202)),
    );
    painter.text(
        rect.left_top() + egui::vec2(8.0, 6.0),
        egui::Align2::LEFT_TOP,
        card.rank.label(),
        egui::FontId::proportional(19.0),
        ink,
    );
    painter.text(
        rect.left_top() + egui::vec2(8.0, 27.0),
        egui::Align2::LEFT_TOP,
        card.suit.symbol().to_string(),
        egui::FontId::proportional(16.0),
        ink,
    );
    painter.text(
        rect.center() + egui::vec2(0.0, 5.0),
        egui::Align2::CENTER_CENTER,
        card.suit.symbol().to_string(),
        egui::FontId::proportional(34.0),
        ink,
    );
}

fn animation_progress(index: usize, animation: Option<CardAnimation>, now: f64) -> f32 {
    let Some(animation) = animation else {
        return 1.0;
    };
    if index < animation.first_card || index >= animation.first_card + animation.card_count {
        return 1.0;
    }

    let delay = (index - animation.first_card) as f64 * CARD_STAGGER;
    let progress = ((now - animation.started_at - delay) / CARD_TRAVEL_TIME).clamp(0.0, 1.0);
    let progress = progress as f32;
    1.0 - (1.0 - progress).powi(3)
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1050.0, 780.0])
            .with_min_inner_size([780.0, 640.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Blackjack",
        options,
        Box::new(|_creation_context| Ok(Box::new(BlackjackApp::default()))),
    )
}
