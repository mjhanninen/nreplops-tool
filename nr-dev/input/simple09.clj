;; simple09.clj -- quoted forms

'form
''form
'''form

 ^:foo '''form
' ^:foo ''form
'' ^:foo 'form
''' ^:foo form

 #_ '''foo 'bar
' #_ ''foo 'bar
'' #_ 'foo 'bar
''' #_ foo 'bar

`form
``form
```form

~form
~~form
~~~form

~@form
~@~@form
~@~@~@form
