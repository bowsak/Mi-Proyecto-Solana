
(async () => {
  // 0) Wallet y balance (auto)
  if (!pg?.wallet?.publicKey) {
    throw new Error("No hay wallet activa en Playground.");
  }
  const ownerPubkey = pg.wallet.publicKey;
  console.log("Wallet activa:", ownerPubkey.toBase58());
  const balance = await pg.connection.getBalance(ownerPubkey);
  console.log(`Balance: ${balance / web3.LAMPORTS_PER_SOL} SOL`);

  // 1) Provider y Program desde Playground
  const provider = new anchor.AnchorProvider(
    pg.connection,
    pg.wallet,
    anchor.AnchorProvider.defaultOptions()
  );
  anchor.setProvider(provider);

  const program = pg.program;
  if (!program) {
    throw new Error("pg.program es undefined. Selecciona el IDL de 'floreria' en el panel Program.");
  }
  const PROGRAM_ID = program.programId;

  // 2) PDA: seeds = ["floreria", owner]
  const [floreriaPda] = web3.PublicKey.findProgramAddressSync(
    [Buffer.from("floreria"), ownerPubkey.toBuffer()],
    PROGRAM_ID
  );
  console.log("Floreria PDA:", floreriaPda.toBase58());

  // 3) Listeners de eventos (opcional)
  const subs = [];
  subs.push(await program.addEventListener("ProductoAgregado", (e, slot) => {
    console.log("Evento ProductoAgregado:", { nombre: e.nombre, precio: e.precio.toString(), stock: e.stock, slot });
  }));
  subs.push(await program.addEventListener("ClienteRegistrado", (e, slot) => {
    console.log("Evento ClienteRegistrado:", { nombre: e.nombre, slot });
  }));
  subs.push(await program.addEventListener("VentaRegistrada", (e, slot) => {
    console.log("Evento VentaRegistrada:", {
      cliente: e.clienteNombre,
      producto: e.productoNombre,
      cantidad: e.cantidad,
      total: e.total.toString(),
      slot,
    });
  }));

  // 4) Datos de prueba
  const NOMBRE_FLORERIA = "Mi Floreria";
  const PRODUCTO_NOMBRE = "Rosas Rojas";
  const PRODUCTO_PRECIO_SOL = 0.001; // 0.001 SOL
  const PRODUCTO_PRECIO_LAMPORTS = new anchor.BN(Math.floor(PRODUCTO_PRECIO_SOL * web3.LAMPORTS_PER_SOL));
  const PRODUCTO_STOCK = 50;
  const CLIENTE_NOMBRE = "Juan";
  const VENTA_CANTIDAD = 2; // u16

  // 5) crear_floreria (crear solo si NO existe) — previene "already in use"
  {
    const accInfo = await pg.connection.getAccountInfo(floreriaPda);
    if (!accInfo) {
      console.log("-- crear_floreria (no existe, creando)");
      const tx = await program.methods
        .crearFloreria(NOMBRE_FLORERIA)
        .accounts({
          owner: ownerPubkey,
          floreria: floreriaPda,
          systemProgram: web3.SystemProgram.programId,
        })
        .rpc();
      console.log("crear_floreria tx:", tx);
    } else {
      console.log("-- crear_floreria: ya existe, continuando...");
    }
  }

  // 6) agregar_producto
  console.log("-- agregar_producto");
  await program.methods
    .agregarProducto(PRODUCTO_NOMBRE, PRODUCTO_PRECIO_LAMPORTS, PRODUCTO_STOCK)
    .accounts({ owner: ownerPubkey, floreria: floreriaPda })
    .rpc();

  // 7) registrar_cliente
  console.log("-- registrar_cliente");
  await program.methods
    .registrarCliente(CLIENTE_NOMBRE)
    .accounts({ owner: ownerPubkey, floreria: floreriaPda })
    .rpc();

  // 8) registrar_venta
  console.log("-- registrar_venta");
  await program.methods
    .registrarVenta(CLIENTE_NOMBRE, PRODUCTO_NOMBRE, VENTA_CANTIDAD)
    .accounts({ owner: ownerPubkey, floreria: floreriaPda })
    .rpc();

  // 9) Leer estado
  const state = await program.account.floreria.fetch(floreriaPda);
  console.log("Resumen estado:", {
    nombre: state.nombre,
    productos: state.productos.length,
    clientes: state.clientes.length,
    primerProducto: state.productos[0]
      ? {
          nombre: state.productos[0].nombre,
          precio: state.productos[0].precio.toString(),
          stock: state.productos[0].stock,
          disponible: state.productos[0].disponible,
        }
      : null,
    primerCliente: state.clientes[0]
      ? {
          nombre: state.clientes[0].nombre,
          compras: state.clientes[0].compras,
          gastadoTotal: state.clientes[0].gastadoTotal.toString(),
        }
      : null,
  });

  // 10) Limpieza de listeners
  for (const id of subs) {
    await program.removeEventListener(id);
  }

  console.log("Listo.");
})().catch((err) => {
  console.error(err);
});
