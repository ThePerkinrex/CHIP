# CHIP

This is a language in which you can define some logic as chips and, using some backend compile into another thing (images, languages, or even a minecraft world backend could be written given enough effort), for now the only backend available is JS.

See the example.chip in src for an example.

That file would be compiled to the folowing js:
```js
class main {
	run(i0,i1,i2){
		let o = false;
		let b = false;
		let AND = new STD_AND();
		let NOT1 = new STD_NOT();
		let r = false;
		o = AND.run((NOT1.run(i0)[0]||i1),i2)[0];
		b = (NOT1.run(i0)[0]||i1);
		return [o,b];
	}
}

class STD_AND {
	run(in0,in1){
		let out = false;
		out = in0 && in1;
		return [out];
	}
}

class STD_NOT {
	run(i){
		let o = false;
		o = !i;
		return [o];
	}
}
```

This is achieved in 1.66 millis with the release compiler. (`cargo build --release`)
