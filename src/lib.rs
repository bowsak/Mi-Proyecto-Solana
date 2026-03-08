/*
FLORERIA

Proposito:
- Gestionar una floreria por owner (PDA unica por propietario).
- Administrar productos y clientes, y registrar ventas simples.
- Acceso: solo el owner puede mutar estado (check en handlers).

Modelo:
- Floreria { owner: Pubkey, nombre: String(max 60), productos: Vec<Producto>(cap 40), clientes: Vec<Cliente>(cap 100) }
- Producto { nombre: String(max 60), precio: u64(lamports), stock: u32, disponible: bool }
- Cliente  { nombre: String(max 60), compras: u32, gastado_total: u64(lamports) }

PDA:
- seeds = ["floreria", owner] (direccion deterministica por propietario)

Instrucciones:
- crear_floreria(owner, nombre) -> crea PDA si no existe
- agregar_producto(owner, nombre, precio, stock)
- eliminar_producto(owner, nombre)
- alterar_producto(owner, nombre)           // toggle disponible
- actualizar_precio(owner, nombre, precio)
- actualizar_stock(owner, nombre, stock)
- registrar_cliente(owner, nombre)
- registrar_venta(owner, cliente_nombre, producto_nombre, cantidad)

Eventos:
- ProductoAgregado, ProductoEliminado, ProductoAlterado
- ProductoPrecioActualizado, ProductoStockActualizado
- ClienteRegistrado, VentaRegistrada

Validaciones clave:
- nombre <= 60, productos <= 40, clientes <= 100
- precio > 0, cantidad > 0, stock suficiente
- total = precio * cantidad con checked_mul (evita overflow)

Nota cliente (Playground):
- Derivar PDA con mismas seeds; antes de crear, hacer getAccountInfo y
  omitir crear_floreria si la cuenta ya existe para evitar "already in use".
*/

use anchor_lang::prelude::*;

declare_id!("34Vrx8KBRxaRmDEFSwVU5n9ZGxhCpHTCWkJoPRJBFUMU");

#[program]
pub mod floreria {
    use super::*;

    /// Crea la florería (PDA).
    pub fn crear_floreria(ctx: Context<NuevaFloreria>, nombre: String) -> Result<()> {
        if nombre.len() > 60 {
            return Err(Errores::NombreMuyLargo.into());
        }

        let owner_id = ctx.accounts.owner.key();
        ctx.accounts.floreria.set_inner(Floreria {
            owner: owner_id,
            nombre,
            productos: Vec::new(),
            clientes: Vec::new(),
        });
        Ok(())
    }

    // -------- Productos --------

    /// Agrega un producto (precio > 0, cap 40).
    pub fn agregar_producto(
        ctx: Context<AccesoFloreria>,
        nombre: String,
        precio: u64,
        stock: u32,
    ) -> Result<()> {
        if ctx.accounts.floreria.owner != ctx.accounts.owner.key() {
            return Err(Errores::NoEresOwner.into());
        }
        if nombre.len() > 60 {
            return Err(Errores::NombreMuyLargo.into());
        }
        if ctx.accounts.floreria.productos.len() >= 40 {
            return Err(Errores::CapacidadDeProductosLlena.into());
        }
        if precio == 0 {
            return Err(Errores::PrecioInvalido.into());
        }

        ctx.accounts.floreria.productos.push(Producto {
            nombre: nombre.clone(),
            precio,
            stock,
            disponible: true,
        });

        let ts = Clock::get()?.unix_timestamp;
        emit!(ProductoAgregado {
            floreria: ctx.accounts.floreria.key(),
            owner: ctx.accounts.owner.key(),
            nombre,
            precio,
            stock,
            timestamp: ts,
        });
        Ok(())
    }

    /// Elimina un producto por nombre.
    pub fn eliminar_producto(ctx: Context<AccesoFloreria>, nombre: String) -> Result<()> {
        if ctx.accounts.floreria.owner != ctx.accounts.owner.key() {
            return Err(Errores::NoEresOwner.into());
        }

        let productos = &mut ctx.accounts.floreria.productos;
        if let Some(idx) = productos.iter().position(|p| p.nombre == nombre) {
            productos.remove(idx);

            let ts = Clock::get()?.unix_timestamp;
            emit!(ProductoEliminado {
                floreria: ctx.accounts.floreria.key(),
                owner: ctx.accounts.owner.key(),
                nombre,
                timestamp: ts
            });
            Ok(())
        } else {
            Err(Errores::ProductoNoExiste.into())
        }
    }

    /// Alterna disponibilidad del producto.
    pub fn alterar_producto(ctx: Context<AccesoFloreria>, nombre: String) -> Result<()> {
        if ctx.accounts.floreria.owner != ctx.accounts.owner.key() {
            return Err(Errores::NoEresOwner.into());
        }

        let productos = &mut ctx.accounts.floreria.productos;
        if let Some(idx) = productos.iter().position(|p| p.nombre == nombre) {
            let nuevo = !productos[idx].disponible;
            productos[idx].disponible = nuevo;

            let ts = Clock::get()?.unix_timestamp;
            emit!(ProductoAlterado {
                floreria: ctx.accounts.floreria.key(),
                owner: ctx.accounts.owner.key(),
                nombre,
                disponible: nuevo,
                timestamp: ts
            });
            Ok(())
        } else {
            Err(Errores::ProductoNoExiste.into())
        }
    }

    /// Actualiza precio (lamports > 0).
    pub fn actualizar_precio(
        ctx: Context<AccesoFloreria>,
        nombre: String,
        nuevo_precio: u64,
    ) -> Result<()> {
        if ctx.accounts.floreria.owner != ctx.accounts.owner.key() {
            return Err(Errores::NoEresOwner.into());
        }
        if nuevo_precio == 0 {
            return Err(Errores::PrecioInvalido.into());
        }

        let productos = &mut ctx.accounts.floreria.productos;
        if let Some(p) = productos.iter_mut().find(|p| p.nombre == nombre) {
            p.precio = nuevo_precio;

            let ts = Clock::get()?.unix_timestamp;
            emit!(ProductoPrecioActualizado {
                floreria: ctx.accounts.floreria.key(),
                owner: ctx.accounts.owner.key(),
                nombre,
                precio: nuevo_precio,
                timestamp: ts
            });
            Ok(())
        } else {
            Err(Errores::ProductoNoExiste.into())
        }
    }

    /// Establece el stock (u32).
    pub fn actualizar_stock(
        ctx: Context<AccesoFloreria>,
        nombre: String,
        nuevo_stock: u32,
    ) -> Result<()> {
        if ctx.accounts.floreria.owner != ctx.accounts.owner.key() {
            return Err(Errores::NoEresOwner.into());
        }

        let productos = &mut ctx.accounts.floreria.productos;
        if let Some(p) = productos.iter_mut().find(|p| p.nombre == nombre) {
            p.stock = nuevo_stock;

            let ts = Clock::get()?.unix_timestamp;
            emit!(ProductoStockActualizado {
                floreria: ctx.accounts.floreria.key(),
                owner: ctx.accounts.owner.key(),
                nombre,
                stock: nuevo_stock,
                timestamp: ts
            });
            Ok(())
        } else {
            Err(Errores::ProductoNoExiste.into())
        }
    }

    // -------- Clientes --------

    /// Registra un cliente (cap 40).
    pub fn registrar_cliente(ctx: Context<AccesoFloreria>, nombre: String) -> Result<()> {
        if ctx.accounts.floreria.owner != ctx.accounts.owner.key() {
            return Err(Errores::NoEresOwner.into());
        }
        if nombre.len() > 60 {
            return Err(Errores::NombreMuyLargo.into());
        }
        if ctx.accounts.floreria.clientes.len() >= 40 {
            return Err(Errores::CapacidadDeClientesLlena.into());
        }

        ctx.accounts.floreria.clientes.push(Cliente {
            nombre: nombre.clone(),
            compras: 0,
            gastado_total: 0,
        });

        let ts = Clock::get()?.unix_timestamp;
        emit!(ClienteRegistrado {
            floreria: ctx.accounts.floreria.key(),
            owner: ctx.accounts.owner.key(),
            nombre,
            timestamp: ts
        });
        Ok(())
    }

    // -------- Ventas --------

    /// Registra venta (baja stock y acumula metricas).
pub fn registrar_venta(
    ctx: Context<AccesoFloreria>,
    cliente_nombre: String,
    producto_nombre: String,
    cantidad: u16,
) -> Result<()> {
    // Control de acceso
    if ctx.accounts.floreria.owner != ctx.accounts.owner.key() {
        return Err(Errores::NoEresOwner.into());
    }
    if cantidad == 0 {
        return Err(Errores::CantidadInvalida.into());
    }

    // Capturar llaves por valor (no mantiene prestamos)
    let floreria_key = ctx.accounts.floreria.key();
    let owner_key = ctx.accounts.owner.key();

    // Buscar indices sin tomar prestamos mutables largos
    let cliente_idx = ctx
        .accounts
        .floreria
        .clientes
        .iter()
        .position(|c| c.nombre == cliente_nombre)
        .ok_or(Errores::ClienteNoExiste)?;

    let producto_idx = ctx
        .accounts
        .floreria
        .productos
        .iter()
        .position(|p| p.nombre == producto_nombre)
        .ok_or(Errores::ProductoNoExiste)?;

    // Chequeos con prestamo inmutable corto
    let precio_unitario: u64 = {
        let p_ref = &ctx.accounts.floreria.productos[producto_idx];
        if !p_ref.disponible {
            return Err(Errores::ProductoNoExiste.into());
        }
        if p_ref.stock < cantidad as u32 {
            return Err(Errores::StockInsuficiente.into());
        }
        p_ref.precio
    };

    // Calculo de total (con overflow checks)
    let total_u128 = (precio_unitario as u128)
        .checked_mul(cantidad as u128)
        .ok_or(Errores::MontoOverflow)?;
    if total_u128 > u64::MAX as u128 {
        return Err(Errores::MontoOverflow.into());
    }
    let total = total_u128 as u64;

    // Mutar producto en un bloque separado
    {
        let p_mut = &mut ctx.accounts.floreria.productos[producto_idx];
        p_mut.stock = p_mut
            .stock
            .checked_sub(cantidad as u32)
            .ok_or(Errores::StockInsuficiente)?;
    }

    // Mutar cliente en otro bloque (prestamo anterior ya termino)
    {
        let c_mut = &mut ctx.accounts.floreria.clientes[cliente_idx];
        c_mut.compras = c_mut.compras.saturating_add(1);
        c_mut.gastado_total = c_mut.gastado_total.saturating_add(total);
    }

    // Evento (usa llaves capturadas y valores escalares)
    let ts = Clock::get()?.unix_timestamp;
    emit!(VentaRegistrada {
        floreria: floreria_key,
        owner: owner_key,
        cliente_nombre,
        producto_nombre,
        cantidad,
        precio_unitario,
        total,
        timestamp: ts
    });

    Ok(())
}

    // -------- Lecturas --------

    /// Log: total de productos.
    pub fn leer_productos(ctx: Context<LecturaFloreria>) -> Result<()> {
        let count = ctx.accounts.floreria.productos.len();
        msg!("Total de productos: {}", count);
        Ok(())
    }

    /// Log: total de clientes.
    pub fn leer_clientes(ctx: Context<LecturaFloreria>) -> Result<()> {
        let count = ctx.accounts.floreria.clientes.len();
        msg!("Total de clientes: {}", count);
        Ok(())
    }
}

#[error_code]
pub enum Errores {
    #[msg("no eres el propietario de esa cuenta")]
    NoEresOwner,
    #[msg("no existe ese producto")]
    ProductoNoExiste,
    #[msg("no existe ese cliente")]
    ClienteNoExiste,
    #[msg("capacidad máxima de productos alcanzada")]
    CapacidadDeProductosLlena,
    #[msg("capacidad máxima de clientes alcanzada")]
    CapacidadDeClientesLlena,
    #[msg("el nombre excede el máximo permitido")]
    NombreMuyLargo,
    #[msg("precio inválido")]
    PrecioInvalido,
    #[msg("stock insuficiente")]
    StockInsuficiente,
    #[msg("cantidad inválida")]
    CantidadInvalida,
    #[msg("overflow en el monto calculado")]
    MontoOverflow,
}

#[account]
#[derive(InitSpace)]
pub struct Floreria {
    /// Autoridad.
    pub owner: Pubkey,
    /// Nombre ≤ 60.
    #[max_len(60)]
    pub nombre: String,
    /// Productos (cap 40).
    #[max_len(40)]
    pub productos: Vec<Producto>,
    /// Clientes (cap 40).
    #[max_len(40)]
    pub clientes: Vec<Cliente>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace, PartialEq, Debug)]
pub struct Producto {
    /// Nombre ≤ 60.
    #[max_len(60)]
    pub nombre: String,
    /// Precio (lamports).
    pub precio: u64,
    /// Existencias.
    pub stock: u32,
    /// Disponible o no.
    pub disponible: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace, PartialEq, Debug)]
pub struct Cliente {
    /// Nombre ≤ 60.
    #[max_len(60)]
    pub nombre: String,
    /// # compras.
    pub compras: u32,
    /// Total gastado (lamports).
    pub gastado_total: u64,
}

#[derive(Accounts)]
pub struct NuevaFloreria<'info> {
    /// Paga e inicializa.
    #[account(mut)]
    pub owner: Signer<'info>,

    /// PDA de florería.
    #[account(
        init,
        payer = owner,
        space = Floreria::INIT_SPACE + 8,
        seeds = [b"floreria", owner.key().as_ref()],
        bump
    )]
    pub floreria: Account<'info, Floreria>,

    pub system_program: Program<'info, System>,
}

/// Mutaciones (control por handler).
#[derive(Accounts)]
pub struct AccesoFloreria<'info> {
    pub owner: Signer<'info>,
    #[account(mut)]
    pub floreria: Account<'info, Floreria>,
}

/// Solo lectura.
#[derive(Accounts)]
pub struct LecturaFloreria<'info> {
    pub floreria: Account<'info, Floreria>,
}

#[event]
pub struct ProductoAgregado {
    pub floreria: Pubkey,
    pub owner: Pubkey,
    pub nombre: String,
    pub precio: u64,
    pub stock: u32,
    pub timestamp: i64,
}

#[event]
pub struct ProductoEliminado {
    pub floreria: Pubkey,
    pub owner: Pubkey,
    pub nombre: String,
    pub timestamp: i64,
}

#[event]
pub struct ProductoAlterado {
    pub floreria: Pubkey,
    pub owner: Pubkey,
    pub nombre: String,
    pub disponible: bool,
    pub timestamp: i64,
}

#[event]
pub struct ProductoPrecioActualizado {
    pub floreria: Pubkey,
    pub owner: Pubkey,
    pub nombre: String,
    pub precio: u64,
    pub timestamp: i64,
}

#[event]
pub struct ProductoStockActualizado {
    pub floreria: Pubkey,
    pub owner: Pubkey,
    pub nombre: String,
    pub stock: u32,
    pub timestamp: i64,
}

#[event]
pub struct ClienteRegistrado {
    pub floreria: Pubkey,
    pub owner: Pubkey,
    pub nombre: String,
    pub timestamp: i64,
}

#[event]
pub struct VentaRegistrada {
    pub floreria: Pubkey,
    pub owner: Pubkey,
    pub cliente_nombre: String,
    pub producto_nombre: String,
    pub cantidad: u16,
    pub precio_unitario: u64,
    pub total: u64,
    pub timestamp: i64,
}
