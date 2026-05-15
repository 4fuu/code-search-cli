package demo;

public class Parser implements Parseable {
    public static final int MAX_SIZE = 42;
    int bufferSize = MAX_SIZE;

    public Parser() {}

    public static Parser create() {
        return new Parser();
    }

    public void parse() {
        helper();
    }

    void helper() {}
}

public interface Parseable {
    void parse();
}

public enum Level {
    INFO,
    DEBUG
}

public record Token(String value) {}
