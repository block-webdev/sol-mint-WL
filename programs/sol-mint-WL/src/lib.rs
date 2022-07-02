use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::{invoke, invoke_signed};
use anchor_spl::token;
use anchor_spl::token::{MintTo, Token};
use std::mem::size_of;
use mpl_token_metadata::instruction::{create_master_edition_v3, create_metadata_accounts_v2};

declare_id!("D3y1bCPeP8fXvc8Ji2pqhS9f8JWmHCcr6rCJE2ZNSzUG");

#[program]
pub mod sol_mint_wl {
    use super::*;

    pub const GLOBAL_STATE_SEED: &[u8] = b"GLOBAL_STATE_SEED1";
    pub const USER_STATE_SEED: &[u8] = b"USER_STATE_SEED1";
    pub const IPFS_METADATA_SEED: &[u8] = b"IPFS_METADATA_SEED";
    pub const NFT_CREATOR_SEED: &str = "NFT_CREATOR_SEED";

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.global_state.admin = ctx.accounts.admin.key();
        ctx.accounts.global_state.user_count = 0;

        Ok(())
    }

    pub fn set_global_state(ctx: Context<SetGlobalState>, wl_limit: u32, wl_price: u64, wl_type: u8) -> Result<()> {
        if wl_type == 0 {
            ctx.accounts.global_state.og_limit = wl_limit;
            ctx.accounts.global_state.og_price = wl_price;
        } else {// if wl_limit == 1 {
            ctx.accounts.global_state.wl_limit = wl_limit;
            ctx.accounts.global_state.wl_price = wl_price;
        }

        Ok(())
    }

    pub fn set_current_wl_type(ctx: Context<SetGlobalState>, wl_type: u8) -> Result<()> {
        ctx.accounts.global_state.cur_wl_type = wl_type;

        Ok(())
    }

    pub fn init_user_state(ctx: Context<InitUserState>, wl_type: u8) -> Result<()> {
        ctx.accounts.user_state.user = ctx.accounts.user.key();
        ctx.accounts.user_state.wl_type = wl_type;
        ctx.accounts.global_state.user_count += 1;

        Ok(())
    }

    pub fn close_user_state(ctx : Context<CloseUserState>) -> Result<()> {
        ctx.accounts.global_state.user_count -= 1;
        Ok(())
    }

    pub fn init_ipfs_metadata(ctx: Context<InitIpfsMetadata>, json_id: u64, json_link: String) -> Result<()> {
        let mut json_data = ctx.accounts.ipfs_metadata.load_init()?;

        json_data.json_id = json_id;
        json_data.json_link[..json_link.len()].copy_from_slice(json_link.as_bytes());

        ctx.accounts.global_state.total_nft_count += 1;

        Ok(())
    }

    pub fn close_ipfs_metadata(_ctx : Context<CloseIpfsMetadata>) -> Result<()> {

        Ok(())
    }

    pub fn str_test(_ctx: Context<StrTest>, uri: String) -> Result<()> {
        let hello = String::from("hello1");

        require!(
            hello == uri,
            NftMintError::InvalidMetadataUri
        );

        Ok(())
    }

    pub fn mint_nft(
        ctx: Context<MintNft>,
        uri: String,
        name: String,
        symbol: String,
    ) -> Result<()> {
        require!(uri.len() != 0, NftMintError::InvalidMetadataUri);

        msg!("Initializing Mint NFT");
        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.acc_token.to_account_info(),
            authority: ctx.accounts.payer.to_account_info(),
        };
        msg!("CPI Accounts Assigned");
        let cpi_program = ctx.accounts.token_program.to_account_info();
        msg!("CPI Program Assigned");
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        msg!("CPI Context Assigned");
        token::mint_to(cpi_ctx, 1)?;
        msg!("Token Minted !!!");

        let maker = &ctx.accounts.maker;

        let tmp_acc = vec![
            ctx.accounts.metadata.to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.mint_owner.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.token_metadata_program.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.rent.to_account_info(),
            maker.to_account_info(),
        ];
        msg!("Account Info Assigned");
        let creator = vec![
            mpl_token_metadata::state::Creator {
                address: maker.key(),
                verified: true,
                share: 100,
            },
            mpl_token_metadata::state::Creator {
                address: ctx.accounts.mint_owner.key(),
                verified: false,
                share: 0,
            },
        ];
        msg!("Creator Assigned");

        let (_creator, creator_bump) =
        Pubkey::find_program_address(&[NFT_CREATOR_SEED.as_bytes()], ctx.program_id);
        let authority_seeds = [NFT_CREATOR_SEED.as_bytes(), &[creator_bump]];

        invoke_signed(
            &create_metadata_accounts_v2(
                ctx.accounts.token_metadata_program.key(),
                ctx.accounts.metadata.key(),
                ctx.accounts.mint.key(),
                ctx.accounts.mint_owner.key(),
                ctx.accounts.payer.key(),
                maker.key(),
                name,
                symbol,
                uri,
                Some(creator),
                1,
                true,
                false,
                None,
                None,
            ),
            tmp_acc.as_slice(),
            &[&authority_seeds],
        )?;
        msg!("Metadata Account Created !!!");
        let master_info = vec![
            ctx.accounts.acc_master.to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.mint_owner.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.metadata.to_account_info(),
            ctx.accounts.token_metadata_program.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.rent.to_account_info(),
            maker.to_account_info(),
        ];
        msg!("Master Edition Account Infos Assigned");
        invoke_signed(
            &create_master_edition_v3(
                ctx.accounts.token_metadata_program.key(),
                ctx.accounts.acc_master.key(),
                ctx.accounts.mint.key(),
                maker.key(),
                ctx.accounts.mint_owner.key(),
                ctx.accounts.metadata.key(),
                ctx.accounts.payer.key(),
                Some(0),
            ),
            master_info.as_slice(),
            &[&authority_seeds],
        )?;
        msg!("Master Edition Nft Minted !!!");

        let mut ipfs_metadata = ctx.accounts.ipfs_metadata.load_mut()?;
        ipfs_metadata.is_minted = 1;

        ctx.accounts.global_state.minted_nft_count += 1;

        Ok(())
    }

}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        seeds = [GLOBAL_STATE_SEED],
        bump,
        payer = admin,
        space = 8 + size_of::<GlobalState>()
    )]
    pub global_state: Account<'info, GlobalState>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    pub system_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct SetGlobalState<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_STATE_SEED],
        bump,
        constraint = global_state.admin.key() == admin.key()
    )]
    pub global_state: Account<'info, GlobalState>,
}

#[derive(Accounts)]
pub struct InitUserState<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_STATE_SEED],
        bump,
        constraint = global_state.admin.key() == admin.key()
    )]
    pub global_state: Account<'info, GlobalState>,

    #[account(
        init,
        seeds = [USER_STATE_SEED, user.key().as_ref()],
        bump,
        payer = admin,
        space = 8 + size_of::<UserState>()
    )]
    pub user_state: Account<'info, UserState>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    // #[account(mut)]
    pub user : AccountInfo<'info>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    pub system_program: AccountInfo<'info>,
}


#[derive(Accounts)]
pub struct CloseUserState<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_STATE_SEED],
        bump,
        constraint = global_state.admin.key() == admin.key()
    )]
    pub global_state: Account<'info, GlobalState>,

    #[account(
        mut,
        close = admin,
    )]
    pub user_state: Account<'info, UserState>,
}

#[derive(Accounts)]
#[instruction(json_id : u64)]
pub struct InitIpfsMetadata<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_STATE_SEED],
        bump,
        constraint = global_state.admin.key() == admin.key()
    )]
    pub global_state: Account<'info, GlobalState>,

    #[account(
        init,
        seeds = [IPFS_METADATA_SEED, &json_id.to_le_bytes()],
        bump,
        payer = admin,
        space = 8 + size_of::<IpfsMetadataState>()
    )]
    pub ipfs_metadata: AccountLoader<'info, IpfsMetadataState>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    pub system_program: AccountInfo<'info>,
}


#[derive(Accounts)]
pub struct CloseIpfsMetadata<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_STATE_SEED],
        bump,
        constraint = global_state.admin.key() == admin.key()
    )]
    pub global_state: Account<'info, GlobalState>,

    #[account(
        mut,
        close = admin,
    )]
    pub ipfs_metadata: AccountLoader<'info, IpfsMetadataState>,
}


#[account]
#[derive(Default)]
pub struct GlobalState {
    // user
    pub admin: Pubkey,
    pub user_count: u32,

    pub total_nft_count: u32,
    pub minted_nft_count: u32,

    pub og_limit: u32,
    pub og_price: u64,

    pub wl_limit: u32,
    pub wl_price: u64,

    pub cur_wl_type: u8, // 0 : OG, 1 : WL, 2 : public
}


#[account]
#[derive(Default)]
pub struct UserState {
    // user
    pub user: Pubkey,
    pub wl_type: u8, // 0 : OG, 1 : WL
}

#[account(zero_copy)]
#[repr(packed)]
pub struct IpfsMetadataState {
    pub json_id: u64,
    pub is_minted: u8,
    pub json_link: [u8; 100], // String
}

impl Default for IpfsMetadataState {
    #[inline]
    fn default() -> IpfsMetadataState {
        IpfsMetadataState {
            json_link: [0; 100],
            json_id: 0,
            is_minted: 0,
        }
    }
}

#[derive(Accounts)]
pub struct StrTest {
}



#[derive(Accounts)]
pub struct MintNft<'info> {
    #[account(mut)]
    pub mint_owner: Signer<'info>,
/// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub mint: UncheckedAccount<'info>,
    // #[account(mut)]
    pub token_program: Program<'info, Token>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub acc_token: UncheckedAccount<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_metadata_program: UncheckedAccount<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub payer: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub rent: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub acc_master: UncheckedAccount<'info>,

    /// CHECK: account constraints checked in account trait
    #[account(mut)]
    pub maker: UncheckedAccount<'info>,

    #[account(mut)]
    pub ipfs_metadata: AccountLoader<'info, IpfsMetadataState>,

    #[account(
        mut,
        seeds = [GLOBAL_STATE_SEED],
        bump,
    )]
    pub global_state: Account<'info, GlobalState>,
}


#[error_code]
pub enum NftMintError {
    #[msg("Invalid metadata uri")]
    InvalidMetadataUri
}