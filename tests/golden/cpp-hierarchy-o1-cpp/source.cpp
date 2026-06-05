/* dac --target cpp -O1 reconstruction
   input: tests/fixtures/cpp-hierarchy-x86_64
   arch:  x86-64
   classes: 5 (polymorphic: 4) members: 8 free: 9 */
#include <cstdint>
#include <cstddef>

// dac-recovered class
// qualified: Animal
// vtable: false
// typeinfo: true
// bases: (none)
// confidence: 1.00 (Observed)
class Animal {
public:
};

// dac-recovered class
// qualified: Cat
// vtable: true
// typeinfo: true
// bases: Public Animal
// confidence: 1.00 (Observed)
class Cat : public Animal {
public:
    // dtor variants:
    //   0x11f2  _ZN3CatD0Ev
    //   0x11e6  _ZN3CatD1Ev
    //   0x11e6  _ZN3CatD2Ev
    virtual ~Cat() {
        // dac C++ stub: lifter→SSA bridge pending; body intentionally empty
    }

    // dac-recovered member
    // address: 0x11dc
    // mangled: _ZNK3Cat5speakEv
    // const:   true
    // virtual: true
    virtual std::int32_t speak() const {
        // dac C++ stub: lifter→SSA bridge pending; body intentionally empty
        return std::int32_t{};
    }
};

// dac-recovered class
// qualified: Dog
// vtable: true
// typeinfo: true
// bases: Public Animal
// confidence: 1.00 (Observed)
class Dog : public Animal {
public:
    // dtor variants:
    //   0x11e8  _ZN3DogD0Ev
    //   0x11e4  _ZN3DogD1Ev
    //   0x11e4  _ZN3DogD2Ev
    virtual ~Dog() {
        // dac C++ stub: lifter→SSA bridge pending; body intentionally empty
    }

    // dac-recovered member
    // address: 0x11d6
    // mangled: _ZNK3Dog5speakEv
    // const:   true
    // virtual: true
    virtual std::int32_t speak() const {
        // dac C++ stub: lifter→SSA bridge pending; body intentionally empty
        return std::int32_t{};
    }
};

namespace __cxxabiv1 {
    // dac-recovered class
    // qualified: __cxxabiv1::__class_type_info
    // vtable: true
    // typeinfo: false
    // bases: (none)
    // confidence: 1.00 (Observed)
    class __class_type_info {
    public:
        // synthesised by dac: vtable present, dtor not in symbol table
        virtual ~__class_type_info() {
            // dac C++ stub: lifter→SSA bridge pending; body intentionally empty
        }
    };
} // namespace __cxxabiv1

namespace __cxxabiv1 {
    // dac-recovered class
    // qualified: __cxxabiv1::__si_class_type_info
    // vtable: true
    // typeinfo: false
    // bases: (none)
    // confidence: 1.00 (Observed)
    class __si_class_type_info {
    public:
        // synthesised by dac: vtable present, dtor not in symbol table
        virtual ~__si_class_type_info() {
            // dac C++ stub: lifter→SSA bridge pending; body intentionally empty
        }
    };
} // namespace __cxxabiv1

// dac-recovered free function
// address: 0x1000
// mangled: _init
void _init() {
    // dac C++ stub: lifter→SSA bridge pending; body intentionally empty
}

// dac-recovered free function
// address: 0x1050
// mangled: main
std::int32_t main() {
    // dac C++ stub: lifter→SSA bridge pending; body intentionally empty
    return std::int32_t{};
}

// dac-recovered free function
// address: 0x10c0
// mangled: _start
void _start() {
    // dac C++ stub: lifter→SSA bridge pending; body intentionally empty
}

// dac-recovered free function
// address: 0x10f0
// mangled: deregister_tm_clones
void deregister_tm_clones() {
    // dac C++ stub: lifter→SSA bridge pending; body intentionally empty
}

// dac-recovered free function
// address: 0x1120
// mangled: register_tm_clones
void register_tm_clones() {
    // dac C++ stub: lifter→SSA bridge pending; body intentionally empty
}

// dac-recovered free function
// address: 0x1160
// mangled: __do_global_dtors_aux
void __do_global_dtors_aux() {
    // dac C++ stub: lifter→SSA bridge pending; body intentionally empty
}

// dac-recovered free function
// address: 0x11b0
// mangled: frame_dummy
void frame_dummy() {
    // dac C++ stub: lifter→SSA bridge pending; body intentionally empty
}

// dac-recovered free function
// address: 0x11b9
// mangled: _Z6chorusPK6AnimalS1_
void chorus() {
    // dac C++ stub: lifter→SSA bridge pending; body intentionally empty
}

// dac-recovered free function
// address: 0x11fc
// mangled: _fini
void _fini() {
    // dac C++ stub: lifter→SSA bridge pending; body intentionally empty
}
