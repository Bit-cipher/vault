#![allow(unexpected_cfgs)]
#![allow(deprecated)]

use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

declare_id!("GDsbXzEgAfXJN6VDC5GWYCjM74m6Cp1ZmLMJ4Q8Qgbsf");

#[program]
pub mod vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.initialize(ctx.bumps)?;
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        ctx.accounts.deposit(amount)?;
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        ctx.accounts.withdraw(amount)?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init,
        payer = user,
        seeds = [b"state", user.key().as_ref()],
        space = 8 + VaultState::INIT_SPACE,
        bump
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn initialize(&mut self, bumps: InitializeBumps) -> Result<()> {
        let rent_exempt: u64 =
            Rent::get()?.minimum_balance(self.vault_state.to_account_info().data_len());

        let cpi_program = self.system_program.to_account_info();

        let cpi_accounts = Transfer {
            from: self.user.to_account_info(),
            to: self.vault.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        transfer(cpi_ctx, rent_exempt)?;

        self.vault_state.vault_bump = bumps.vault;
        self.vault_state.state_bump = bumps.vault_state;
        self.vault_state.authority = self.user.key();

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"state", user.key().as_ref()],
        bump = vault_state.state_bump
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump
    )]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> Deposit<'info> {
    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        let cpi_program = self.system_program.to_account_info();

        let cpi_accounts = Transfer {
            from: self.user.to_account_info(),
            to: self.vault.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        transfer(cpi_ctx, amount)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"state", user.key().as_ref()],
        bump = vault_state.state_bump
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump
    )]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(&mut self, amount: u64) -> Result<()> {
        let rent = Rent::get()?;
        let min_rent_balance = rent.minimum_balance(self.vault_state.to_account_info().data_len());

        // Ensure vault has enough SOL and stays rent-exempt
        require!(
            self.vault.to_account_info().lamports() >= amount + min_rent_balance,
            VaultError::InsufficientBalance
        );

        let cpi_program = self.system_program.to_account_info();

        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.user.to_account_info(),
        };

        let vault_state_key = self.vault_state.key();
        let seeds: &[&[u8]] = &[
            b"vault",
            vault_state_key.as_ref(),
            &[self.vault_state.vault_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        transfer(cpi_ctx, amount)?;

        Ok(())
    }
}


#[account]
#[derive(InitSpace)]
pub struct VaultState {
    pub vault_bump: u8,
    pub state_bump: u8,
    pub authority: Pubkey,
}


#[derive(Accounts)]
pub struct CloseVault<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        close = authority,
        has_one = authority
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(mut)]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}


#[error_code]
pub enum VaultError {
    #[msg("Insufficient balance in vault for withdrawal.")]
    InsufficientBalance,
}
