
describe("floreria e2e", () => {
  const NOMBRE_FLORERIA = "Mi Floreria Test";
  const PRODUCTO_NOMBRE = "Tulipanes";
  const PRODUCTO_PRECIO_SOL = 0.002;
  const PRODUCTO_STOCK = 10;
  const CLIENTE_NOMBRE = "Ana";
  const VENTA_CANTIDAD = 2;

  it("flujo completo", async () => {
    // 0) Wallet y provider
    if (!pg?.wallet?.publicKey) throw new Error("No hay wallet activa en Playground.");
    const ownerPubkey = pg.wallet.publicKey;

    const provider = new anchor.AnchorProvider(
      pg.connection,
      pg.wallet,
      anchor.AnchorProvider.defaultOptions()
    );
    anchor.setProvider(provider);

    // 1) Programa
    const program = pg.program;
    if (!program) throw new Error("Selecciona el IDL de floreria en el panel Program.");

    // 2) PDA
    const [floreriaPda] = web3.PublicKey.findProgramAddressSync(
      [Buffer.from("floreria"), ownerPubkey.toBuffer()],
      program.programId
    );

    // 3) crear_floreria (crear solo si NO existe)
    {
      const accInfo = await pg.connection.getAccountInfo(floreriaPda);
      if (!accInfo) {
        await program.methods
          .crearFloreria(NOMBRE_FLORERIA)
          .accounts({
            owner: ownerPubkey,
            floreria: floreriaPda,
            systemProgram: web3.SystemProgram.programId,
          })
          .rpc();
      }
    }

    // 4) agregar_producto
    const precioLamports = new anchor.BN(Math.floor(PRODUCTO_PRECIO_SOL * web3.LAMPORTS_PER_SOL));
    await program.methods
      .agregarProducto(PRODUCTO_NOMBRE, precioLamports, PRODUCTO_STOCK)
      .accounts({ owner: ownerPubkey, floreria: floreriaPda })
      .rpc();

    // 5) registrar_cliente
    await program.methods
      .registrarCliente(CLIENTE_NOMBRE)
      .accounts({ owner: ownerPubkey, floreria: floreriaPda })
      .rpc();

    // 6) registrar_venta
    await program.methods
      .registrarVenta(CLIENTE_NOMBRE, PRODUCTO_NOMBRE, VENTA_CANTIDAD)
      .accounts({ owner: ownerPubkey, floreria: floreriaPda })
      .rpc();

    // 7) Verificar estado
    const state = await program.account.floreria.fetch(floreriaPda);
    if (!state) throw new Error("No se pudo leer la cuenta Floreria");
    if (state.nombre !== NOMBRE_FLORERIA && state.nombre !== "Mi Floreria") {
      throw new Error("Nombre inesperado de la floreria: " + state.nombre);
    }

    if (state.productos.length < 1) throw new Error("Sin productos");
    const p0 = state.productos[0];
    if (p0.nombre !== PRODUCTO_NOMBRE) throw new Error("Producto incorrecto");
    if (Number(p0.precio) !== Number(precioLamports)) throw new Error("Precio no coincide");
    if (p0.stock !== PRODUCTO_STOCK - VENTA_CANTIDAD) throw new Error("Stock no se actualizo tras la venta");

    if (state.clientes.length < 1) throw new Error("Sin clientes");
    const c0 = state.clientes[0];
    if (c0.nombre !== CLIENTE_NOMBRE) throw new Error("Cliente incorrecto");
    if (c0.compras !== 1) throw new Error("Compras no acumularon");
    if (Number(c0.gastadoTotal) <= 0) throw new Error("gastadoTotal invalido");

    console.log("OK: flujo completo");
  });
});
