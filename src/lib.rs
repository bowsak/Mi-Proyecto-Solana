use anchor_lang::prelude::*;

declare_id!("DVPqCHGKdqNyMdfUBgmZ5Giewb4cz6tSHE4nvzGMkGNd");

#[program]
pub mod biblioteca {
    use super::*;

    pub fn crear_biblioteca(context: Context<NuevaBiblioteca>, nombre: String) -> Result<()> {
        let owner_id = context.accounts.owner.key();
        let libros: Vec<Libro> = Vec::new();

        context.accounts.biblioteca.set_inner(Biblioteca {
            owner: owner_id,
            nombre,
            libros,
        });

        Ok(())
    }

    pub fn agregar_libro(context: Context<NuevoLibro>, nombre: String, paginas: u16) -> Result<()> {
        let libro = Libro {
            nombre,
            paginas,
            disponible: true,
        };

        context.accounts.biblioteca.libros.push(libro);

        Ok(())
    }

    pub fn leer_libro(context: Context<NuevoLibro>) -> Result<()> {
        msg!("La lista de libros es: {:#?}", context.accounts.biblioteca.libros);
        Ok(())
    }

    pub fn eliminar_libro(context: Context<NuevoLibro>, nombre: String) -> Result<()> {
        let libros = &mut context.accounts.biblioteca.libros;

        for i in 0..libros.len() {
            if libros[i].nombre == nombre {
                libros.remove(i);
                msg!("libro eliminado");
                return Ok(());
            }
        }

        Err(Errores::LibroNoExiste.into())
    }

    pub fn alterar_libro(context: Context<NuevoLibro>, nombre: String) -> Result<()> {
        let libros = &mut context.accounts.biblioteca.libros;

        for i in 0..libros.len() {
            if libros[i].nombre == nombre {
                let estado = libros[i].disponible;
                libros[i].disponible = !estado;
                return Ok(());
            }
        }

        Err(Errores::LibroNoExiste.into())
    }
}

#[error_code]
pub enum Errores {
    #[msg("no eres el propietario de esa cuenta")]
    NoEresOwner,
    #[msg("no existe ese libro")]
    LibroNoExiste,
}

#[account]
#[derive(InitSpace)]
pub struct Biblioteca {
    pub owner: Pubkey,

    #[max_len(60)]
    pub nombre: String,

    #[max_len(10)]
    pub libros: Vec<Libro>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace, PartialEq, Debug)]
pub struct Libro {
    #[max_len(60)]
    pub nombre: String,

    pub paginas: u16,

    pub disponible: bool,
}

#[derive(Accounts)]
pub struct NuevaBiblioteca<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        init,
        payer = owner,
        space = Biblioteca::INIT_SPACE + 8,
        seeds = [b"biblioteca", owner.key().as_ref()],
        bump
    )]
    pub biblioteca: Account<'info, Biblioteca>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct NuevoLibro<'info> {
    pub owner: Signer<'info>,

    #[account(mut)]
    pub biblioteca: Account<'info, Biblioteca>,
}
