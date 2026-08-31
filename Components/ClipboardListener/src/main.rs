use wayland_clipboard_listener::WlClipboardPasteStream;
//Es la estructura principal (struct) de la biblioteca. Se encarga de conectarse al servidor gráfico (Wayland) y 
//abstraer la complejidad subyacente de los protocolos wlr-data-control-unstable-v1 y ext-data-control-v1.
use wayland_clipboard_listener::WlListenType;
//Es un enumerador (enum) que le permite al programador definir el comportamiento exacto del bucle de escucha.

fn main() {
    let mut stream = WlClipboardPasteStream::init(WlListenType::ListenOnCopy).unwrap();
    //Init: Configura y establece la conexión inicial con el compositor de Wayland
    //WlListenType::ListenOnCopy: : Le indica a la herramienta que no debe extraer el contenido actual del
        //portapapeles de inmediato, sino suscribirse pasivamente y reaccionar solo cuando ocurra un evento
    //unwrap(): "si ocurre un error al inicializar (por ejemplo, si no estás ejecutando Wayland o tu 
        //compositor no soporta el protocolo), interrumpe el proceso inmediatamente con un panic".
    for context in stream.paste_stream().flatten(){
        //paste_stream(): Inicia el bucle de eventos bloqueante que 
            // mantiene la ejecución de tu demonio viva a la espera de iteraciones.
        //Al aplicar flatten(), el iterador ignora automáticamente los errores transitorios y 
            //extrae (desenvuelve) únicamente las capturas que fueron exitosas, simplificando tu código
        println!("{context:?}");
    }

    //Ejemplo de output:
    /*
    {
    mime_types: [
        "chromium/x-web-custom-data",
        "chromium/x-internal-source-rfh-token",
        "text/plain;charset=utf-8",
        "text/plain",
        "text/plain;charset=utf-8",
        "UTF8_STRING",
        "STRING",
        "TEXT",
        "text/html",
        "chromium/x-source-url"
    ],
    context: ClipBoardListenContext {
        mime_type: "text/plain;charset=utf-8",
        context: [101, 97, 109, 40, 41, 46, 102, 108, 97, 116, 116]
    }
}
    */

}
