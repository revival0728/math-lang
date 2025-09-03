CC = g++
CPPFLAGS = -std=c++17
OUT_DIR = ./build
SRC_DIR = ./src
TARGET = math-lang
OBJS = main.o compiler.o utils.o runtime.o interpreter.o

OUT_OBJS = $(addprefix $(OUT_DIR)/, $(OBJS))

bin: ${OUT_DIR}/$(TARGET)

all: CPPFLAGS += -O3
all: bin

debug: CPPFLAGS += -g -Wall -DDEBUG
debug: bin

$(OUT_DIR)/$(TARGET): $(OUT_OBJS)
	$(CC) $(CPPFLAGS) -o $@ $^

$(OUT_DIR)/%.o: $(SRC_DIR)/%.cpp
	@mkdir -p $(OUT_DIR)
	$(CC) $(CPPFLAGS) -c -o $@ $<

clean:
	rm -rf $(OUT_DIR)/*.o

.PHONY: all clean debug bin

