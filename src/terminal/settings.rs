use crate::{settings::Settings, token::Token, Terminal};
use console::Term;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Password, Select};
use ethers::prelude::k256::elliptic_curve::Error;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

pub struct SettingsScreen {}

#[derive(FromPrimitive)]
enum SettingsTopics {
    Hops,
    Slippage,
    External,
    Back,
}

impl SettingsScreen {
    pub fn render() -> std::io::Result<()> {
        let topics = [
            "1. Path hops",
            "2. Slippage tolerance",
            "3. Compare with external quote",
            "<- Go back",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .items(&topics)
            .default(0)
            .interact_on_opt(&Term::stderr())?;

        match selection {
            Some(index) => match FromPrimitive::from_usize(index) {
                Some(SettingsTopics::Hops) => {
                    let max_steps = Self::select_max_steps();
                    Settings::set_max_steps(max_steps);

                    Terminal::render();
                }
                Some(SettingsTopics::Slippage) => {
                    let slippage = Self::select_slippage();
                    Settings::set_slippage(slippage);

                    Terminal::render();
                }
                Some(SettingsTopics::External) => {
                    let is_allowed = Self::confirm_is_external_allowed();
                    Settings::set_is_external_allowed(is_allowed);

                    Terminal::render();
                }
                Some(SettingsTopics::Back) => {
                    Terminal::render();
                }
                None => panic!("Error while selecting token screen topic"),
            },
            None => println!("You did not select anything"),
        }

        Ok(())
    }

    fn select_max_steps() -> i32 {
        let current_max_steps = Settings::get_max_steps();

        let max_steps_items = vec![1, 2, 3, 4];

        let steps_select_index = max_steps_items
            .iter()
            .position(|&i| i == current_max_steps)
            .unwrap();

        let max_steps_selection = Select::with_theme(&ColorfulTheme::default())
            .items(&max_steps_items)
            .with_prompt("How many steps you want to search the path with (recommended 3, because of view-gas limitations)")
            .default(steps_select_index)
            .interact_on_opt(&Term::stderr())
            .unwrap();

        let max_steps = max_steps_items[max_steps_selection.unwrap()];

        max_steps
    }

    fn select_slippage() -> u32 {
        let current_slippage = Settings::get_slippage();

        let slippage_items = vec![1, 5, 10];
        let slippage_items_format = vec!["0.1%", "0.5%", "1%"];

        let slippage_select_index = slippage_items
            .iter()
            .position(|&i| i == current_slippage)
            .unwrap();

        let slippage_selection = Select::with_theme(&ColorfulTheme::default())
            .items(&slippage_items_format)
            .with_prompt("Select slippage tolerance")
            .default(slippage_select_index)
            .interact_on_opt(&Term::stderr())
            .unwrap();

        let slippage = slippage_items[slippage_selection.unwrap()];

        slippage
    }

    fn confirm_is_external_allowed() -> bool {
        let is_external_allowed = Settings::is_external_allowed();

        let confirm_is_allowed = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Do you want to compare price with 1inch?")
            .default(is_external_allowed)
            .interact()
            .unwrap();

        confirm_is_allowed
    }
}
