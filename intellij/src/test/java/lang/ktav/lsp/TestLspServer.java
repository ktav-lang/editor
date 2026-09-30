package lang.ktav.lsp;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;

import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.util.HashMap;
import java.util.Map;

/** A real stdio child with a gated formatting response, not a mocked client/transport. */
public final class TestLspServer {
  public static void main(String[] args) throws Exception {
    Path gate = Path.of(args[0]);
    boolean silentInitialize = Boolean.parseBoolean(args[1]);
    Map<String, String> documents = new HashMap<>();
    while (true) {
      StringBuilder header = new StringBuilder();
      int next;
      while ((next = System.in.read()) != -1) {
        header.append((char)next);
        if (header.toString().endsWith("\r\n\r\n")) break;
      }
      if (next == -1) return;
      int length = Integer.parseInt(header.toString().split("\r\n")[0].split(":", 2)[1].trim());
      JsonObject message = JsonParser.parseString(new String(System.in.readNBytes(length), StandardCharsets.UTF_8)).getAsJsonObject();
      String method = message.get("method").getAsString();
      JsonObject params = message.has("params") && message.get("params").isJsonObject() ? message.getAsJsonObject("params") : null;
      if (method.equals("initialize")) {
        record(gate, "initialize.received", "received");
        if (!silentInitialize) respond(message, new JsonObject());
      } else if (method.equals("textDocument/didOpen")) {
        JsonObject document = params.getAsJsonObject("textDocument");
        documents.put(document.get("uri").getAsString(), document.get("text").getAsString());
      } else if (method.equals("textDocument/didChange")) {
        String uri = params.getAsJsonObject("textDocument").get("uri").getAsString();
        documents.put(uri, params.getAsJsonArray("contentChanges").get(0).getAsJsonObject().get("text").getAsString());
        record(gate, "change.received", documents.get(uri));
      } else if (method.equals("textDocument/didClose")) {
        documents.remove(params.getAsJsonObject("textDocument").get("uri").getAsString());
      } else if (method.equals("textDocument/formatting")) {
        String text = documents.get(params.getAsJsonObject("textDocument").get("uri").getAsString());
        if (text == null) throw new IllegalStateException("Formatting without an open document");
        record(gate, "format.received", text);
        while (!Files.exists(gate.resolve("format.release"))) Thread.sleep(10);
        JsonObject edit = JsonParser.parseString("{\"range\":{\"start\":{\"line\":0,\"character\":0},\"end\":{\"line\":0,\"character\":0}}}").getAsJsonObject();
        int lastNewline = text.lastIndexOf('\n');
        JsonObject end = edit.getAsJsonObject("range").getAsJsonObject("end");
        end.addProperty("line", text.chars().filter(c -> c == '\n').count());
        end.addProperty("character", text.length() - lastNewline - 1);
        edit.addProperty("newText", text.replace("v:1", "v: 1"));
        JsonArray edits = new JsonArray();
        edits.add(edit);
        respond(message, edits);
      } else if (method.equals("shutdown")) {
        respond(message, null);
      } else if (method.equals("exit")) return;
    }
  }

  private static void record(Path gate, String name, String text) throws Exception {
    Path temporary = gate.resolve(name + ".tmp");
    Files.writeString(temporary, text);
    Files.move(temporary, gate.resolve(name), StandardCopyOption.ATOMIC_MOVE, StandardCopyOption.REPLACE_EXISTING);
  }

  private static void respond(JsonObject request, com.google.gson.JsonElement result) throws Exception {
    JsonObject response = new JsonObject();
    response.addProperty("jsonrpc", "2.0");
    response.add("id", request.get("id"));
    response.add("result", result);
    byte[] body = response.toString().getBytes(StandardCharsets.UTF_8);
    System.out.write(("Content-Length: " + body.length + "\r\n\r\n").getBytes(StandardCharsets.US_ASCII));
    System.out.write(body);
    System.out.flush();
  }
}
