module Logging
  VERSION = "1.0"
  current_level = VERSION

  class Logger
    def self.build(name)
      Logger.new(name)
    end

    def initialize(name)
      @name = name
    end

    def log(message)
      puts message
    end
  end
end
