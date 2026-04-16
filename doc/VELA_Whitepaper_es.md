# Libro Blanco de Equidad y Seguridad del Contrato VELA

> Este documento está dirigido a todos los usuarios de la comunidad VELA, con el objetivo de explicar de manera integral el diseño y las garantías del contrato inteligente VELA en términos de seguridad, equidad y transparencia.

---

## 1. Auditoría Autorizada de Terceros

El contrato inteligente de VELA ha sido sometido a una rigurosa auditoría por parte de **CertiK**, la firma de auditoría de seguridad blockchain número uno del mundo. CertiK ha proporcionado servicios de auditoría de seguridad a más de 4,000 proyectos blockchain, incluyendo proyectos reconocidos como Aave, Polygon y BNB Chain, siendo la institución de auditoría de seguridad más autorizada reconocida por la industria.

La auditoría abarcó:
- Revisión integral de la lógica del código del contrato
- Escaneo de vulnerabilidades comunes (ataques de reentrada, desbordamientos, elusión de permisos, etc.)
- Detección de puertas traseras y verificación de seguridad de fondos
- Confirmación de la seguridad y fiabilidad de la lógica de flujo de fondos

Conclusión de la auditoría: El código del contrato no presenta vulnerabilidades de seguridad, ni puertas traseras ni funciones ocultas de retiro por parte del administrador.

El informe de auditoría está disponible públicamente: **https://skynet.certik.com/projects/vela**

Cualquier persona puede consultar la puntuación de seguridad y los detalles de la auditoría de VELA en el sitio web oficial de CertiK.

---

## 2. Código Completamente de Código Abierto

VELA cree firmemente en que "el código es ley", y hemos hecho público todo nuestro código, abierto a la supervisión de toda la sociedad:

**Código fuente del contrato inteligente**
- El código del contrato está alojado en GitHub, accesible para que cualquier persona lo revise
- Dirección del repositorio: https://github.com/veladex/vela-anchor

**Código del sitio web frontend**
- Para disipar las preocupaciones de los usuarios sobre la seguridad del sitio web, también hemos hecho de código abierto el código frontend empaquetado
- Dirección del repositorio: https://github.com/veladex/vela-website

Esto significa que VELA no tiene ninguna "caja negra" — desde el contrato en cadena hasta la interfaz web que usted utiliza, todo es transparente.

---

## 3. Verificación de Código en Cadena — Ver para Creer

¿Es realmente el código abierto idéntico al programa que se ejecuta en cadena? No necesita confiar en nuestra palabra, puede verificarlo usted mismo.

Solana proporciona la herramienta oficial de verificación de código `solana-verify`, que permite a cualquier persona ejecutar el siguiente comando para comparar byte por byte el código fuente en GitHub con el programa desplegado en cadena:

```bash
solana-verify verify-from-repo \
    --url https://api.mainnet-beta.solana.com \
    --program-id FW6P7G9yPBqGAGsZ6Aa7upC9whF69QMH4ZJaBJjFsLVK \
    https://github.com/veladex/vela-anchor
```

Resultado de la verificación:

```
Executable Program Hash from repo: 5a9221595cb0f03287e8f58c1613173316ff036f274668f4db716d8eab2fc343
On-chain Program Hash: 5a9221595cb0f03287e8f58c1613173316ff036f274668f4db716d8eab2fc343
Program hash matches ✅
```

Los dos valores hash son completamente idénticos, lo que demuestra que el programa en cadena es exactamente igual al resultado compilado del código abierto, sin ninguna manipulación.

---

## 4. Código del Contrato Verificable

Después del despliegue del contrato, el código queda fijado en la blockchain de Solana y genera un hash de programa único. Este hash ha sido certificado por la auditoría de CertiK y registrado públicamente. Esto significa:

- Cualquier modificación al código del contrato produciría un hash de programa completamente diferente, que no pasaría la verificación de auditoría de CertiK
- Incluso si se redespliega con el mismo código fuente, se generaría una nueva dirección de programa del contrato, que tampoco coincidiría con los registros auditados
- Cualquier miembro de la comunidad puede comparar en cualquier momento el programa en cadena con el código abierto usando la herramienta `solana-verify` para confirmar la consistencia

Cada una de sus operaciones de staking y cada retiro de ganancias se ejecuta estrictamente según las reglas escritas en el código en cadena, con la garantía continua de la auditoría de CertiK.

---

## 5. Seguridad de Fondos — Nadie Puede Tocar Su Dinero

Esta es la pregunta que más preocupa a los usuarios, y damos la respuesta más clara:

**No existe ninguna función en el contrato que permita al administrador transferir los fondos de los usuarios.**

- Sus fondos se almacenan en cuentas dedicadas (PDA) en la blockchain, controladas por el programa del contrato
- Todas las operaciones de fondos (staking, redención, retiro de ganancias) requieren la firma de su propia billetera
- Los permisos del administrador se limitan a la configuración de inicialización (como crear colecciones de NFT, inicializar el sistema de referidos), completamente ajenos al flujo de fondos
- El 10% de impuesto deducido al reclamar intereses se envía directamente a la dirección de quema de Solana (dirección de agujero negro), sin entrar en la billetera de nadie

Incluso si el sitio web de VELA se cierra, sus fondos permanecen seguros en la blockchain y pueden ser recuperados en cualquier momento a través de la interacción en cadena. Para métodos detallados de recuperación de fondos, consulte la "Guía de Seguridad de Fondos".

---

## 6. Reglas Justas y Transparentes

### Reglas de Staking Públicas y Transparentes

| Período de Staking | Tasa Diaria | Rango de Staking |
|---------|--------|---------|
| 7 días | 0.5% | 1,000 - 50,000 VELA |
| 30 días | 0.7% | 1,000 - 50,000 VELA |
| 90 días | 1.0% | 1,000 - 50,000 VELA |

Las tasas anteriores están codificadas de forma fija en el código del contrato, son completamente iguales para todos los usuarios, sin ningún "trato especial" ni "reglas ocultas".

### Destino de Impuestos Transparente

El 10% de impuesto deducido al reclamar intereses se envía al 100% a la dirección de quema en la cadena de Solana. Este proceso se ejecuta completamente en cadena, y cualquier persona puede verificarlo en tiempo real a través de exploradores de bloques (como Solscan, Solana Explorer).

### Cantidad Limitada y Pública de NFTs

- Suministro máximo de Diamond NFT: 600 unidades
- Suministro máximo de Gold NFT: 12,000 unidades

Los límites de suministro están codificados de forma fija en el código del contrato y no pueden ser aumentados.

---

## 7. Sin Dependencia del Sitio Web — Verdadera Descentralización

VELA es un protocolo descentralizado completamente desplegado en la blockchain de Solana. El sitio web es simplemente una interfaz conveniente para sus operaciones; el contrato en sí funciona de forma independiente en la blockchain.

Proporcionamos scripts de interacción de código abierto que le permiten interactuar directamente con el contrato para completar todas las operaciones, incluso sin pasar por el sitio web. El código del script se encuentra en `scripts/stake-example.js` del repositorio del proyecto, y se utiliza de la siguiente manera:

1. Instalar el entorno Node.js (versión v16 o superior)
2. Clonar el repositorio de código e instalar dependencias
   ```bash
   git clone https://github.com/veladex/vela-anchor.git
   cd vela-anchor/scripts
   npm install
   ```
3. Configurar su información en `stake-example.js`:
   - Reemplazar `privateKeyString` con la clave privada de su billetera (formato bs58)
   - Cambiar la dirección RPC de `connection` al RPC de la red principal de Solana (por ejemplo, `https://api.mainnet-beta.solana.com`)
   - Cambiar `mintAddress` a la dirección del token VELA en la red principal
4. Llamar a la función correspondiente según sea necesario:
   ```bash
   node stake-example.js
   ```

El script contiene ejemplos completos de operaciones:

| Nombre de Función | Funcionalidad | Descripción |
|--------|------|------|
| `addReferral` | Vincular referente | Debe vincular un referente antes de hacer staking |
| `createStake` | Crear staking | Especificar monto y tipo de período (1=7 días, 2=30 días, 3=90 días) |
| `getMyStakingOrders` | Consultar órdenes de staking | Ver todos los stakings en curso |
| `claimInterest` | Reclamar intereses | Reclamar los intereses acumulados de una orden específica |
| `unstake` | Desstaking | Redimir el capital y los intereses restantes después del vencimiento |

Todas las operaciones solo requieren la firma de su propia billetera, sin necesidad de autorización de terceros.

> Nota: Si no está familiarizado con las operaciones técnicas, puede pedir a cualquier desarrollador de la comunidad que le ayude a verificar y ejecutar — ya que el código es completamente de código abierto, cualquier persona puede confirmar la seguridad del script.

---

## 8. Datos en Cadena Públicamente Consultables

Todos los registros de transacciones, flujos de fondos y estados del contrato de VELA están registrados en la blockchain de Solana, y cualquier persona puede consultarlos a través de las siguientes herramientas:

- **Solana Explorer**: https://explorer.solana.com
- **Solscan**: https://solscan.io

Puede consultar en cualquier momento:
- Todas las transacciones históricas del contrato
- El saldo en tiempo real del pool de fondos
- Cada registro de quema de impuestos
- Su estado personal de staking y ganancias

---

## 9. Preguntas Frecuentes

**P: ¿Podría el equipo del proyecto huir con los fondos?**
R: No. No existe ninguna función de retiro de administrador en el contrato; los fondos solo pueden ser operados por el propio usuario a través de la firma de su billetera. Incluso si el equipo se disuelve, sus fondos permanecen seguros en la cadena y pueden ser recuperados en cualquier momento.

**P: ¿Podría el contrato ser actualizado secretamente para añadir una puerta trasera?**
R: No. El código del contrato ha sido auditado por CertiK y ha generado un hash de programa único. Cualquier modificación del código cambiaría el hash, que no pasaría la verificación de auditoría; incluso redesplegar con el código fuente produciría una nueva dirección de programa, que tampoco sería reconocida por CertiK. Puede verificar independientemente la consistencia entre el código en cadena y el repositorio de código abierto en cualquier momento usando la herramienta `solana-verify`.

**P: ¿Podrían las tasas de interés ser modificadas en secreto?**
R: No. Los parámetros de tasa de interés están codificados de forma fija en el código del contrato, no son variables modificables. Para cambiar las tasas se requeriría desplegar un contrato completamente nuevo (nueva dirección), y los stakings en el contrato antiguo no se verían afectados.

**P: ¿Es justo el cálculo de mis ganancias por referidos?**
R: La lógica de cálculo de ganancias por referidos está completamente escrita en el contrato en cadena, tratando a todos por igual. Puede revisar el código abierto o el informe de auditoría para verificar las reglas de cálculo.

**P: ¿Podrían los NFTs ser emitidos infinitamente, diluyendo su valor?**
R: No. El límite de Diamond NFT es de 600 unidades y el de Gold NFT es de 12,000 unidades; estos números están codificados de forma fija en el contrato y no pueden ser modificados.

**P: Si no entiendo de tecnología, ¿cómo puedo confirmar que estas afirmaciones son verdaderas?**
R: Puede pedir a cualquier amigo que entienda de desarrollo en Solana que le ayude a verificar, o consultar el informe de auditoría de CertiK. Nuestro código es completamente de código abierto y resiste la inspección de cualquier persona.

**P: ¿Podría el contrato ser hackeado?**
R: El contrato VELA ha sido auditado profesionalmente por CertiK, la número uno del mundo. El código está desarrollado con el framework Anchor, el más maduro del ecosistema Solana, siguiendo los más altos estándares de seguridad de la industria. Además, el código es completamente de código abierto y está continuamente bajo el escrutinio de investigadores de seguridad de todo el mundo.

**P: ¿Podría el sitio web robar mi clave privada?**
R: No. El contrato y el sitio web de VELA no almacenan ni acceden a su clave privada. Todas las firmas de transacciones se completan localmente en su billetera (como Phantom). El código frontend también es de código abierto, y cualquier persona puede verificarlo. Asegúrese de guardar bien su clave privada o frase mnemónica, ya que es la única garantía de la seguridad de sus fondos.

---

## Resumen

| Dimensión de Seguridad | Enfoque de VELA |
|---------|------------|
| Seguridad del Código | Auditoría autorizada de CertiK, número uno del mundo, cero vulnerabilidades y cero puertas traseras |
| Transparencia del Código | Contrato + frontend completamente de código abierto |
| Autenticidad del Código | Verificación en cadena con solana-verify, hashes coincidentes |
| Contrato Verificable | Hash bloqueado por auditoría de CertiK, cualquier cambio es detectable inmediatamente |
| Autonomía de Fondos | Control por firma de billetera del usuario, sin retiros de administrador |
| Reglas Justas | Tasas de interés, tasas de impuestos, límites de NFT todos codificados de forma fija |
| Impuestos Transparentes | 10% de impuesto quemado directamente, verificable en cadena |
| Descentralización | Sin dependencia del sitio web, scripts de código abierto para interacción directa |

*La seguridad de VELA no se basa en promesas, sino en el código, la auditoría y la transparencia de la blockchain. Damos la bienvenida al escrutinio y la verificación de cada usuario e investigador de seguridad.*
